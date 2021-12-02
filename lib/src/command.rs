pub trait Actor<TCommand, TResponse> {
    fn execute(&mut self, command: TCommand) -> TResponse;
}

pub trait UndoActor<TCommand, TResponse> {
    fn undo(&mut self, command: TCommand) -> TResponse;
}
