use std::{path::Path, pin::Pin, time::Duration};

use anyhow::Result;
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
    time::sleep,
};

use crate::distro;

pub(crate) async fn streaming_print(mut stream: Pin<Box<dyn AsyncBufRead + Send>>, prefix: &str) {
    let mut buffer = [0; 1024];
    while let Ok(n) = stream.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        print!("{prefix}: {}", String::from_utf8_lossy(&buffer[..n]));
    }
}

pub(crate) async fn setup_mongo_container() -> Result<ContainerAsync<Mongo>> {
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

pub(crate) async fn server_health_check() {
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
                        "Error: Server did not start within {} seconds.",
                        timeout_secs
                    );
                    return;
                }
                println!("Waiting for server...");
                sleep(Duration::from_secs(5)).await;
                elapsed += 5;
            }
        }
    }
}

pub(crate) async fn setup_server() -> Result<ContainerAsync<GenericImage>> {
    let container = GenericBuildableImage::new("packhub-test-server", "latest")
        .with_dockerfile("images/server-ci.Dockerfile")
        .with_file("./scripts/run_server.sh", "run_server.sh")
        .with_file("../src", "src")
        .with_file("../pages", "pages")
        .with_file("../templates", "templates")
        .with_file("../data/self-signed-certs", "data/self-signed-certs")
        .with_file("../.env", ".env")
        .with_file("../Cargo.toml", "Cargo.toml")
        // .with_file("../Cargo.lock", "Cargo.lock")
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

pub(crate) async fn setup_distro(distro: distro::Distro) -> Result<distro::Distro> {
    let mut container = GenericImage::new(&distro.image, &distro.tag)
        .with_entrypoint(&format!("./{}", &distro.script))
        .with_platform("linux/amd64")
        .with_copy_to(
            &distro.script,
            Path::new(&format!("scripts/{}", &distro.script)),
        )
        .with_network("host");

    if let Some(ref env) = distro.env {
        for (key, value) in env {
            container = container.with_env_var(key, value);
        }
    }

    let container = container.start().await?;

    let stdout = container.stdout(true);

    let name = format!("{}({}:{})", &distro.name, &distro.image, &distro.tag);

    let name2 = name.clone();

    tokio::spawn(async move {
        streaming_print(stdout, &name).await;
    });

    let stderr = container.stderr(true);

    tokio::spawn(async move {
        streaming_print(stderr, &name2).await;
    });

    loop {
        let Some(code) = container.exit_code().await? else {
            continue;
        };

        if code == 0 {
            return Ok(distro);
        } else {
            return Err(anyhow::Error::new(distro::DistroError { distro, code }));
        }
    }
}
