use std::{path::Path, pin::Pin, process::ExitCode, time::Duration};

use anyhow::{Result, anyhow, bail};
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{
        ContainerAsync, GenericBuildableImage, GenericImage, ImageExt,
        core::IntoContainerPort,
        runners::{AsyncBuilder, AsyncRunner},
    },
};
use tokio::{
    io::{AsyncBufRead, AsyncReadExt},
    task::JoinHandle,
    time::sleep,
};

mod logger;

struct InfraTest {
    mongo: ContainerAsync<Mongo>,
    server: ContainerAsync<GenericImage>,
    distro: Vec<ContainerAsync<GenericImage>>,
}

// async fn logger(container: &ContainerAsync<GenericImage>, ) {
//     let mut stream = container.logs().await.unwrap();
//     streaming_print(stream).await;
// }

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => bail!("handling failed: {err}"),
    }
}

impl InfraTest {
    async fn setup_infra() -> Result<Self> {
        let handle1 = tokio::spawn(setup_mongo_container());
        let handle2 = tokio::spawn(setup_server());

        let Ok((mongo, server)) = tokio::try_join!(flatten(handle1), flatten(handle2)) else {
            bail!("Failed to setup infrastructure");
        };

        Ok(Self {
            mongo,
            server,
            distro: Vec::new(),
        })
    }
}

async fn streaming_print(mut stream: Pin<Box<dyn AsyncBufRead + Send>>, prefix: &str) {
    let mut buffer = [0; 1024];
    while let Ok(n) = stream.read(&mut buffer).await {
        if n == 0 {
            println!("{prefix}: ################### Stream ended prematurely #################");
            break;
        }
        print!("{prefix}: {}", String::from_utf8_lossy(&buffer[..n]));
    }
}

async fn setup_mongo_container() -> Result<ContainerAsync<Mongo>> {
    let container = Mongo::default()
        .with_network("host")
        .with_mapped_port(27017, 27017.tcp())
        .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
        .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "pass")
        .start()
        .await?;

    println!("MongoDB container started");

    Ok(container)
}

async fn server_health_check() {
    let timeout_secs = 60;
    let mut elapsed = 0;

    let client = reqwest::Client::new();

    loop {
        let resp = client.get("http://localhost:3000").send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                println!("Server is up!");
                break;
            }
            _ => {
                if elapsed >= timeout_secs {
                    println!(
                        "Server did not start within {} seconds. But continuing...",
                        timeout_secs
                    );
                    // exit 0 (like the shell script: continue-on-error)
                    return;
                }
                println!("Waiting for server...");
                sleep(Duration::from_secs(5)).await;
                elapsed += 5;
            }
        }
    }
}

async fn setup_server() -> Result<ContainerAsync<GenericImage>> {
    let container = GenericBuildableImage::new("packhub-test-server", "latest")
        .with_dockerfile("images/server-ci-2.Dockerfile")
        .with_file("./scripts/run_server.sh", "run_server.sh")
        .with_file("../src", "src")
        .with_file("../pages", "pages")
        .with_file("../templates", "templates")
        .with_file("../data/self-signed-certs", "data/self-signed-certs")
        .with_file("../.env", ".env")
        .with_file("../Cargo.toml", "Cargo.toml")
        // .with_data("../Cargo.lock", "Cargo.lock")
        .build_image()
        .await?
        // .with_wait_for(WaitFor::http(
        //     HttpWaitStrategy::new("/") // GET /
        //         .with_port(3000.tcp()) // use mapped host port for 3000
        //         .with_expected_status_code(200_u16) // treat 200 as “up”
        //         .with_poll_interval(Duration::from_secs(5)),
        // ))
        // .with_startup_timeout(Duration::from_secs(60)) // total timeout ~ your TIMEOUT
        .with_network("host")
        .with_mapped_port(3000, 3000.tcp())
        .start()
        .await?;

    // println!("after await\n");

    // println!("Container logs:\n");

    // let stdout = container.stdout(true);

    // streaming_print(stdout).await;
    server_health_check().await;

    // println!("Server Container logs:\n");
    let stdout = container.stdout(true);
    tokio::spawn(async move {
        streaming_print(stdout, "server").await;
    });

    // println!("\nServer Container error logs:\n");
    let stderr = container.stderr(true);

    tokio::spawn(async move {
        streaming_print(stderr, "server").await;
    });

    // let e = container.exit_code().await?;
    // println!("\nServer Container exit code: {:?}", e);

    Ok(container)
}

async fn setup_distro() -> Result<ContainerAsync<GenericImage>> {
    let container = GenericImage::new("ubuntu", "24.04")
        .with_entrypoint("./check_apt.sh")
        // .with_entrypoint("./run_distro.sh")
        .with_platform("linux/amd64")
        .with_env_var("DIST", "ubuntu")
        .with_copy_to("check_apt.sh", Path::new("scripts/check_apt.sh"))
        // .with_copy_to("run_distro.sh", Path::new("scripts/run_distro.sh"))
        .with_network("host")
        .start()
        .await?;

    // let mut buffer = String::new();
    // container.stdout(true).read_to_string(&mut buffer).await?;
    // container.stderr(true).read_to_string(&mut buffer).await?;

    // println!("Container logs:\n{}", buffer);

    println!("Distribution Container logs:\n");
    let stdout = container.stdout(true);
    streaming_print(stdout, "ubuntu:24.04 stdout").await;

    println!("\nDistribution Container error logs:\n");
    let stderr = container.stderr(true);
    streaming_print(stderr, "ubuntu:24.04 stderr").await;

    // let e = container.exit_code().await?;
    // println!("\nDistribution Container exit code: {:?}", e);

    Ok(container)
}

#[tokio::main]
async fn main() -> Result<()> {
    // setup_mongo_container().await?;
    // setup_server().await?;
    // setup_distro().await?;
    let infra = InfraTest::setup_infra().await?;
    let container = setup_distro().await?;
    loop {
        let Some(code) = container.exit_code().await? else {
            continue;
        };

        if code == 0 {
            println!("Distribution Container exited successfully");
            break;
        } else {
            println!("Distribution Container exited with error code: {}", code);
            break;
        }
    }
    Ok(())
}
