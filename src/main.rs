use gethostname::gethostname;
use mdns_sd::{Error, ServiceDaemon, ServiceInfo};
use std::net::TcpListener;
use thiserror::Error;
use zero2prod::run;

fn unregister(mdns: &ServiceDaemon, service_fullname: &str) -> Result<(), AppError> {
    match mdns.unregister(service_fullname) {
        Ok(_) => Ok(()),
        Err(err) => match err {
            Error::Again => unregister(mdns, service_fullname),
            e => Err(AppError::from(e)),
        },
    }
}

fn shutdown(mdns: ServiceDaemon) -> Result<(), AppError> {
    match mdns.shutdown() {
        Ok(_) => Ok(()),
        Err(err) => match err {
            Error::Again => shutdown(mdns),
            e => Err(AppError::from(e)),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let tcp_listener = TcpListener::bind("[::]:0").expect("Failed to bind random port");
    let port = tcp_listener.local_addr().unwrap().port();
    let run_result = run(tcp_listener);

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = "_my-test-service._tcp.local.";
    let instance_name = "my_instance";

    let hostname = gethostname().to_str().unwrap().to_string();
    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        &format!("{hostname}.local."),
        "::",
        port,
        None,
    )?
    .enable_addr_auto();

    // Register with the daemon, which publishes the service.
    let service_fullname = my_service.get_fullname().to_string();
    mdns.register(my_service)
        .expect("Failed to register our service");

    run_result?.await.expect("Failed to launch server");

    // Gracefully shutdown the service
    unregister(&mdns, &service_fullname)?;
    shutdown(mdns)
}

#[derive(Debug, Error)]
enum AppError {
    #[error("MDNS error: {0}")]
    Mdns(#[from] mdns_sd::Error),
    #[error("Server error: {0}")]
    Server(#[from] std::io::Error),
}
