use crate::cache;
use runner_pb::runner_server::{Runner, RunnerServer};
use runner_pb::{RunTestRequest, RunTestResponse};
use titanlib_rs::qc_tests::dip_check;
use tonic::{transport::Server, Request, Response, Status};

mod runner_pb {
    tonic::include_proto!("runner");
}

#[derive(Debug, Default)]
struct MyRunner {}

#[tonic::async_trait]
impl Runner for MyRunner {
    #[tracing::instrument]
    async fn run_test(
        &self,
        request: Request<RunTestRequest>,
    ) -> Result<Response<RunTestResponse>, Status> {
        tracing::info!("Got a request: {:?}", request);

        let req = request.into_inner();

        let flag: u32 = match req.test.as_str() {
            "dip_check" => {
                let data = cache::get_timeseries_data(
                    req.data_id,
                    req.time
                        .ok_or_else(|| Status::invalid_argument("invalid timestamp"))?
                        .seconds,
                )
                .await
                .map_err(|err| Status::not_found(format!("data not found by cache: {}", err)))?;
                dip_check(data, 2., 3.) as u32 //TODO use actual test params
            }
            _ => return Err(Status::invalid_argument("invalid test name")),
        };

        let response = RunTestResponse { flag, flag_id: 0 };

        tracing::debug!("sending response");

        Ok(Response::new(response))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = "[::1]:1338".parse()?;
    let runner = MyRunner::default();

    tracing::info!(message = "Starting server.", %addr);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("helloworld_server"))
        .add_service(RunnerServer::new(runner))
        .serve(addr)
        .await?;

    Ok(())
}