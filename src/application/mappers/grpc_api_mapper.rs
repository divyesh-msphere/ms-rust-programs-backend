pub trait GrpcApiMapper<UseCaseOutput, UseCaseInput, Response, Request> {
    fn input(request: Request) -> UseCaseInput;
    fn output_success(output: UseCaseOutput) -> Response;
}
