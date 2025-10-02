use anyhow::{Result, bail};
use testcontainers_modules::{
    mongo::Mongo,
    testcontainers::{ContainerAsync, GenericImage},
};
use tokio::task::{JoinHandle, JoinSet};

use crate::{
    containers::{setup_distro, setup_mongo_container, setup_server},
    distro::{Distro, DistroError},
};

pub struct InfraTest {
    #[allow(dead_code)]
    mongo: ContainerAsync<Mongo>,
    #[allow(dead_code)]
    server: ContainerAsync<GenericImage>,
    distro: JoinSet<Result<Distro>>,
}

pub(crate) async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => bail!("handling failed: {err}"),
    }
}

impl InfraTest {
    pub(crate) async fn setup_infra() -> Result<Self> {
        let handle1 = tokio::spawn(setup_mongo_container());
        let handle2 = tokio::spawn(setup_server());

        let (mongo, server) = match tokio::try_join!(flatten(handle1), flatten(handle2)) {
            Ok((mongo, server)) => (mongo, server),
            Err(err) => bail!("Failed to setup infrastructure: {err}"),
        };

        Ok(Self {
            mongo,
            server,
            distro: JoinSet::new(),
        })
    }

    pub(crate) fn add_distro(&mut self, distro: Distro) {
        self.distro.spawn(async move { setup_distro(distro).await });
    }

    pub(crate) async fn run_distros(&mut self) -> Result<()> {
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
            bail!("Failed to run distros");
        }

        Ok(())
    }
}
