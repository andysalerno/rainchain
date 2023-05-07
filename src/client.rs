pub trait Client<T> {
    fn receive(&self) -> String;
    fn send(&self, message: T);
}

pub struct WebsocketClient {}

impl<T> Client<T> for WebsocketClient {
    fn receive(&self) -> String {
        todo!()
    }

    fn send(&self, _message: T) {
        todo!()
    }
}
