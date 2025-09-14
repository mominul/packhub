use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, WaitContainerOptions,
};
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::models::{ContainerInspectResponse, HostConfig, PortBinding};
use bollard::network::CreateNetworkOptions;
use bollard::service::{BuildInfo, ContainerCreateResponse};
use bollard::{API_DEFAULT_VERSION, Docker};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct TestContainer {
    name: String,
    dockerfile: String,
    context_dir: String,
    image_name: String,
}

#[derive(Debug)]
struct TestResult {
    container_name: String,
    exit_code: i64,
    success: bool,
}

struct LinuxDistributionTestPipeline {
    docker: Docker,
    server_container_id: Option<String>,
    mongodb_container_id: Option<String>,
    test_results: Vec<TestResult>,
}

fn validate_path(path: String) -> Option<String> {
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

impl LinuxDistributionTestPipeline {
    async fn new() -> Result<Self> {
        let home_dir = std::env::home_dir().context("Failed to get home directory")?;
        println!("Home directory: {:?}", home_dir);
        let socket = format!("{}/.docker/run/docker.sock", home_dir.display());
        let docker = Docker::connect_with_unix(&socket, 120, API_DEFAULT_VERSION)
            .context("Failed to connect to Docker")?;

        Ok(Self {
            docker,
            server_container_id: None,
            mongodb_container_id: None,
            test_results: Vec::new(),
        })
    }

    async fn setup_environment(&mut self) -> Result<()> {
        println!("Setting up environment...");

        // Copy .env.example to .env
        // if let Err(e) = fs::copy(".env.example", ".env") {
        //     eprintln!("Warning: Failed to copy .env.example to .env: {}", e);
        // }

        Ok(())
    }

    async fn start_mongodb_service(&mut self) -> Result<()> {
        println!("Starting MongoDB service...");

        // Pull MongoDB image
        let create_image_options = CreateImageOptions {
            from_image: "mongo:5.0.6",
            ..Default::default()
        };

        let mut stream = self
            .docker
            .create_image(Some(create_image_options), None, None);
        while let Some(result) = stream.next().await {
            match result {
                Ok(_) => {}
                Err(e) => eprintln!("Error pulling MongoDB image: {}", e),
            }
        }

        // Create MongoDB container
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "27017/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("27017".to_string()),
            }]),
        );

        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        };

        let mut env_vars = Vec::new();
        env_vars.push("MONGO_INITDB_ROOT_USERNAME=root");
        env_vars.push("MONGO_INITDB_ROOT_PASSWORD=pass");

        let container_config = Config {
            image: Some("mongo:5.0.6"),
            env: Some(env_vars),
            host_config: Some(host_config),
            healthcheck: Some(bollard::models::HealthConfig {
                test: Some(vec!["CMD".to_string(), "mongo".to_string()]),
                interval: Some(10_000_000_000), // 10 seconds in nanoseconds
                timeout: Some(5_000_000_000),   // 5 seconds in nanoseconds
                retries: Some(5),
                ..Default::default()
            }),
            ..Default::default()
        };

        let create_options = CreateContainerOptions {
            name: "mongodb-service",
            platform: Some("linux/amd64"),
        };

        let container = self
            .docker
            .create_container(Some(create_options), container_config)
            .await
            .context("Failed to create MongoDB container")?;

        self.mongodb_container_id = Some(container.id.clone());

        // Start MongoDB container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start MongoDB container")?;

        println!("MongoDB service started");
        Ok(())
    }

    async fn build_image(
        &self,
        dockerfile: &str,
        context_dir: &str,
        image_name: &str,
    ) -> Result<()> {
        use bollard::body_full;
        println!("Building image {} from {}", image_name, dockerfile);

        let build_options = BuildImageOptions {
            dockerfile,
            t: image_name,
            rm: true,
            ..Default::default()
        };

        // Create a tar archive of the context directory
        let tar_data = self.create_tar_archive(context_dir)?;

        let mut stream =
            self.docker
                .build_image(build_options, None, Some(body_full(tar_data.into())));

        while let Some(result) = stream.next().await {
            match result {
                Ok(BuildInfo {
                    stream: Some(output),
                    ..
                }) => {
                    print!("{}", output);
                }
                Ok(BuildInfo {
                    error: Some(error), ..
                }) => {
                    eprintln!("Build error: {}", error);
                    return Err(anyhow::anyhow!("Build failed: {}", error));
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    return Err(e.into());
                }
                _ => {}
            }
        }

        println!("Successfully built image: {}", image_name);
        Ok(())
    }

    fn create_tar_archive(&self, context_dir: &str) -> Result<Vec<u8>> {
        use std::io::Write;
        use tar::Builder;

        let mut tar_data = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_data);
            builder.append_dir_all(".", context_dir)?;
            builder.finish()?;
        }

        Ok(tar_data)
    }

    async fn build_and_run_server(&mut self) -> Result<()> {
        println!("Building and running server container...");

        // Build server image
        self.build_image("images/server-ci.Dockerfile", ".", "server")
            .await?;

        // Create and start server container
        let host_config = HostConfig {
            network_mode: Some("host".to_string()),
            ..Default::default()
        };

        let container_config = Config {
            image: Some("server"),
            host_config: Some(host_config),
            ..Default::default()
        };

        let create_options = CreateContainerOptions {
            name: "server-container",
            platform: Some("linux/amd64"),
        };

        let container = self
            .docker
            .create_container(Some(create_options), container_config)
            .await
            .context("Failed to create server container")?;

        self.server_container_id = Some(container.id.clone());

        // Start server container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start server container")?;

        println!("Server container started");
        Ok(())
    }

    async fn wait_for_server(&self) -> Result<()> {
        println!("Waiting for server to start...");

        let timeout = Duration::from_secs(60);
        let check_interval = Duration::from_secs(5);
        let mut elapsed = Duration::from_secs(0);

        while elapsed < timeout {
            // Try to connect to the server
            match reqwest::get("http://localhost:3000").await {
                Ok(response) if response.status().is_success() => {
                    println!("Server is up!");
                    return Ok(());
                }
                _ => {
                    println!("Waiting for server...");
                    sleep(check_interval).await;
                    elapsed += check_interval;
                }
            }
        }

        println!(
            "Server did not start within {} seconds. But continuing...",
            timeout.as_secs()
        );
        Ok(())
    }

    async fn run_test_container(&mut self, test: &TestContainer) -> Result<TestResult> {
        println!("Running test: {}", test.name);

        // Build test image
        self.build_image(&test.dockerfile, &test.context_dir, &test.image_name)
            .await?;

        // Create and run test container
        let host_config = HostConfig {
            network_mode: Some("host".to_string()),
            ..Default::default()
        };

        let container_config = Config::<&str> {
            image: Some(&test.image_name),
            host_config: Some(host_config),
            ..Default::default()
        };

        let create_options = CreateContainerOptions::<&str> {
            name: &test.name,
            platform: Some("linux/amd64"),
        };

        let container = self
            .docker
            .create_container(Some(create_options), container_config)
            .await
            .context("Failed to create test container")?;

        // Start container
        if let Err(e) = self
            .docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
        {
            eprintln!("Failed to start container {}: {}", test.name, e);
            return Ok(TestResult {
                container_name: test.name.clone(),
                exit_code: -1,
                success: false,
            });
        }

        // Wait for container to finish
        let wait_options = WaitContainerOptions {
            condition: "not-running",
        };

        let mut wait_stream = self
            .docker
            .wait_container(&container.id, Some(wait_options));

        let exit_code = if let Some(result) = wait_stream.next().await {
            match result {
                Ok(wait_result) => wait_result.status_code,
                Err(e) => {
                    eprintln!("Error waiting for container {}: {}", test.name, e);
                    -1
                }
            }
        } else {
            -1
        };

        let success = exit_code == 0;

        println!("Test {} completed with exit code: {}", test.name, exit_code);

        Ok(TestResult {
            container_name: test.name.clone(),
            exit_code,
            success,
        })
    }

    async fn check_server_logs(&self) -> Result<()> {
        if let Some(container_id) = &self.server_container_id {
            println!("Checking server logs...");

            let logs_options = LogsOptions::<String> {
                stdout: true,
                stderr: true,
                ..Default::default()
            };

            let mut logs_stream = self.docker.logs(container_id, Some(logs_options));

            while let Some(log_result) = logs_stream.next().await {
                match log_result {
                    Ok(log_output) => {
                        print!("{}", log_output);
                    }
                    Err(e) => {
                        eprintln!("Error reading server logs: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        println!("Cleaning up containers...");

        let remove_options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        // Remove all test containers
        let test_container_names = [
            "ubuntu24.04-test",
            "fedora42-test",
            "debian12-test",
            "ubuntu24.04-multitest",
            "fedora42-multitest",
            "debian12-multitest",
            "tumbleweed-multitest",
        ];

        for name in &test_container_names {
            if let Err(e) = self
                .docker
                .remove_container(name, Some(remove_options.clone()))
                .await
            {
                eprintln!("Warning: Failed to remove container {}: {}", name, e);
            }
        }

        // Remove server container
        if let Some(container_id) = &self.server_container_id {
            if let Err(e) = self
                .docker
                .remove_container(container_id, Some(remove_options.clone()))
                .await
            {
                eprintln!("Warning: Failed to remove server container: {}", e);
            }
        }

        // Remove MongoDB container
        if let Some(container_id) = &self.mongodb_container_id {
            if let Err(e) = self
                .docker
                .remove_container(container_id, Some(remove_options))
                .await
            {
                eprintln!("Warning: Failed to remove MongoDB container: {}", e);
            }
        }

        Ok(())
    }

    async fn run_all_tests(&mut self) -> Result<()> {
        // Define all test containers
        let test_containers = vec![
            TestContainer {
                name: "ubuntu24.04-test".to_string(),
                dockerfile: "images/ubuntu24.04.Dockerfile".to_string(),
                context_dir: "scripts/".to_string(),
                image_name: "ubuntu24.04".to_string(),
            },
            // TestContainer {
            //     name: "fedora42-test".to_string(),
            //     dockerfile: "images/fedora42.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "fedora42".to_string(),
            // },
            // TestContainer {
            //     name: "debian12-test".to_string(),
            //     dockerfile: "images/debian12.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "debian12".to_string(),
            // },
            // TestContainer {
            //     name: "ubuntu24.04-multitest".to_string(),
            //     dockerfile: "images/ubuntu24.04-multi-package.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "ubuntu24.04-multi".to_string(),
            // },
            // TestContainer {
            //     name: "fedora42-multitest".to_string(),
            //     dockerfile: "images/fedora42-multi-package.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "fedora42".to_string(),
            // },
            // TestContainer {
            //     name: "debian12-multitest".to_string(),
            //     dockerfile: "images/debian12-multi-package.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "debian12-multi".to_string(),
            // },
            // TestContainer {
            //     name: "tumbleweed-multitest".to_string(),
            //     dockerfile: "images/tumbleweed-multi-package.Dockerfile".to_string(),
            //     context_dir: "scripts/".to_string(),
            //     image_name: "tumbleweed-multi".to_string(),
            // },
        ];

        // Run all tests
        for test in &test_containers {
            let result = self.run_test_container(test).await?;
            self.test_results.push(result);
        }

        Ok(())
    }

    fn check_test_results(&self) -> Result<()> {
        println!("Checking test results...");

        let mut failed_tests = Vec::new();

        for result in &self.test_results {
            if !result.success {
                failed_tests.push(result.container_name.clone());

                let error_message = match result.container_name.as_str() {
                    "ubuntu24.04-test" => "Test on Ubuntu 24.04 failed",
                    "fedora42-test" => "Test on Fedora 42 failed",
                    "debian12-test" => "Test on Debian 12 failed",
                    "ubuntu24.04-multitest" => {
                        "Test on Ubuntu 24.04 for multiple package support failed"
                    }
                    "fedora42-multitest" => "Test on Fedora 42 for multiple package support failed",
                    "debian12-multitest" => "Test on Debian 12 for multiple package support failed",
                    "tumbleweed-multitest" => {
                        "Test on OpenSuse Tumbleweed for multiple package support failed"
                    }
                    _ => "Unknown test failed",
                };

                eprintln!("{}", error_message);
            }
        }

        if !failed_tests.is_empty() {
            return Err(anyhow::anyhow!("Tests failed: {:?}", failed_tests));
        }

        println!("All tests passed!");
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        // Setup environment
        self.setup_environment().await?;

        // Start MongoDB service
        self.start_mongodb_service().await?;

        // Build and run server
        self.build_and_run_server().await?;

        // Wait for server to start
        self.wait_for_server().await?;

        // Run all tests
        self.run_all_tests().await?;

        // Check server logs
        self.check_server_logs().await?;

        // Check test results
        self.check_test_results()?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Linux Distribution Test Pipeline...");

    let mut pipeline = LinuxDistributionTestPipeline::new().await?;

    let result = pipeline.run().await;

    // Always cleanup, regardless of test results
    if let Err(e) = pipeline.cleanup().await {
        eprintln!("Cleanup failed: {}", e);
    }

    match result {
        Ok(()) => {
            println!("Pipeline completed successfully!");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Pipeline failed: {}", e);
            process::exit(1);
        }
    }
}
