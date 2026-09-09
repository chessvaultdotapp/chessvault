fn main() -> iced::Result {
    iced::application(ChessVault::default, ChessVault::update, ChessVault::view)
        .title("ChessVault")
        .run()
}

#[derive(Default)]
struct ChessVault;

impl ChessVault {
    fn update(&mut self, _message: ()) {}

    fn view(&self) -> iced::Element<'_, ()> {
        iced::widget::Space::new().into()
    }
}
