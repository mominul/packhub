use std::{path::Path, pin::Pin, time::Duration};

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
    task::{JoinHandle, JoinSet},
    time::sleep,
};

struct InfraTest {
    mongo: ContainerAsync<Mongo>,
    server: ContainerAsync<GenericImage>,
    distro: JoinSet<Result<Distro>>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Distro {
    name: String,
    image: String,
    tag: String,
    env: Option<Vec<(String, String)>>,
    script: String,
}

#[derive(Debug)]
struct DistroError {
    distro: Distro,
    code: i64,
}

impl std::fmt::Display for DistroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Distribution Container {}({}:{}) exited with error code: {}",
            self.distro.name, self.distro.image, self.distro.tag, self.code
        )
    }
}

impl std::error::Error for DistroError {}

impl Distro {
    fn new(name: &str, image: &str, tag: &str, script: &str, env: Option<&[(&str, &str)]>) -> Self {
        Distro {
            name: name.to_owned(),
            image: image.to_owned(),
            tag: tag.to_owned(),
            env: env.map(|env| {
                env.into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<(String, String)>>()
            }),
            script: script.to_owned(),
        }
    }
}

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
            distro: JoinSet::new(),
        })
    }

    fn add_distro(&mut self, distro: Distro) {
        self.distro.spawn(async move { setup_distro(distro).await });
    }

    async fn run_distros(&mut self) -> Result<()> {
        let mut success = Vec::new();
        let mut failed = Vec::new();
        while let Some(result) = self.distro.join_next().await {
            match result {
                Ok(Ok(distro)) => success.push(distro),
                Ok(Err(err)) => {
                    match err.downcast::<DistroError>() {
                        Ok(err) => failed.push(err),
                        Err(_) => bail!("Unexpected error in running distro"),
                    };
                }
                Err(err) => bail!("Distro task failed: {err}"),
            };
        }

        println!();

        for distro in success {
            println!(
                "Successfully ran distro {}({}:{})",
                distro.name, distro.image, distro.tag
            );
        }

        if !failed.is_empty() {
            println!();
            for err in failed {
                println!(
                    "Failed to run distro {}({}:{}) exit code: {}",
                    err.distro.name, err.distro.image, err.distro.tag, err.code
                );
            }
        }

        Ok(())
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

async fn setup_distro(distro: Distro) -> Result<Distro> {
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
            return Err(anyhow::Error::new(DistroError { distro, code }));
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut infra = InfraTest::setup_infra().await?;

    infra.add_distro(Distro::new(
        "single-pkg",
        "ubuntu",
        "24.04",
        "check_apt.sh",
        Some(&[("DIST", "ubuntu")]),
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "ubuntu",
        "24.04",
        "check_apt_multiple.sh",
        Some(&[("DIST", "ubuntu")]),
    ));

    infra.add_distro(Distro::new(
        "single-pkg",
        "debian",
        "12",
        "check_apt.sh",
        Some(&[("DIST", "debian")]),
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "debian",
        "12",
        "check_apt_multiple.sh",
        Some(&[("DIST", "debian")]),
    ));

    infra.add_distro(Distro::new(
        "single-pkg",
        "fedora",
        "42",
        "check_dnf.sh",
        None,
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "fedora",
        "42",
        "check_dnf_multiple.sh",
        None,
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "opensuse/tumbleweed",
        "latest",
        "check_zypper_multiple.sh",
        None,
    ));

    infra.run_distros().await?;

    Ok(())
}
