use std::{collections::HashMap, path::Path, pin::Pin, time::Duration};

use anyhow::{Result, bail};
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

struct Distro {
    image: String,
    tag: String,
    env: Option<HashMap<String, String>>,
    script: String,
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
        .with_dockerfile("images/server-ci.Dockerfile")
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
        .with_network("host")
        .with_mapped_port(3000, 3000.tcp())
        .start()
        .await?;

    server_health_check().await;

    let stdout = container.stdout(true);
    tokio::spawn(async move {
        streaming_print(stdout, "server").await;
    });

    let stderr = container.stderr(true);

    tokio::spawn(async move {
        streaming_print(stderr, "server").await;
    });

    Ok(container)
}

async fn setup_distro(distro: Distro) -> Result<()> {
    let mut container = GenericImage::new(&distro.image, &distro.tag)
        .with_entrypoint(&format!("./{}", &distro.script))
        .with_platform("linux/amd64")
        .with_copy_to(
            &distro.script,
            Path::new(&format!("scripts/{}", &distro.script)),
        )
        .with_network("host");

    if let Some(env) = distro.env {
        for (key, value) in env {
            container = container.with_env_var(key, value);
        }
    }

    let container = container.start().await?;

    let stdout = container.stdout(true);

    tokio::spawn(async move {
        streaming_print(stdout, "ubuntu:24.04").await;
    });

    let stderr = container.stderr(true);

    tokio::spawn(async move {
        streaming_print(stderr, "ubuntu:24.04").await;
    });

    loop {
        let Some(code) = container.exit_code().await? else {
            continue;
        };

        if code == 0 {
            println!(
                "Distribution Container {}:{} exited successfully",
                distro.image, distro.tag
            );
            break;
        } else {
            println!(
                "Distribution Container {}:{} exited with error code: {}",
                distro.image, distro.tag, code
            );
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let infra = InfraTest::setup_infra().await?;

    setup_distro(Distro {
        image: "ubuntu".to_owned(),
        tag: "24.04".to_owned(),
        env: Some([("DIST".to_owned(), "ubuntu".to_owned())].into()),
        script: "check_apt.sh".to_owned(),
    })
    .await?;

    Ok(())
}
