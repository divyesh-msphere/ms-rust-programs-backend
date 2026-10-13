pub trait HttpApiMapper<UseCaseOutput, UseCaseInput, Presenter, Payload> {
    fn input(payload: Payload) -> UseCaseInput;
    fn output_success(output: UseCaseOutput) -> Presenter;
}
