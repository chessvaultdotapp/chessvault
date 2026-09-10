use iced::widget::{Space, column, container, responsive, row, text};
use iced::{Color, Element, Fill};

const LIGHT: Color = Color::from_rgb8(240, 217, 181);
const DARK: Color = Color::from_rgb8(181, 136, 99);

pub fn view<'a, Message: 'a>() -> Element<'a, Message> {
    responsive(|size| {
        let side = size.width.min(size.height);
        let square_size = side / 8.0;
        let label_size = (square_size * 0.22).min(16.0);
        let padding = square_size * 0.06;
        let mut board = column![];

        for rank in (0..8).rev() {
            let mut squares = row![];
            for file in 0..8 {
                // a1 is dark; White's home rank is at the bottom.
                let is_light = (rank + file) % 2 != 0;
                let background = if is_light { LIGHT } else { DARK };
                let foreground = if is_light { DARK } else { LIGHT };
                let rank_label = if file == 0 {
                    (rank + 1).to_string()
                } else {
                    String::new()
                };
                let file_label = if rank == 0 {
                    char::from(b'a' + file as u8).to_string()
                } else {
                    String::new()
                };

                let labels = column![
                    text(rank_label).size(label_size).color(foreground),
                    Space::new().height(Fill),
                    row![
                        Space::new().width(Fill),
                        text(file_label).size(label_size).color(foreground),
                    ],
                ];
                squares = squares.push(
                    container(labels)
                        .padding(padding)
                        .width(square_size)
                        .height(square_size)
                        .style(move |_| container::Style::default().background(background)),
                );
            }
            board = board.push(squares);
        }

        container(board).center_x(Fill).center_y(Fill).into()
    })
    .into()
}
