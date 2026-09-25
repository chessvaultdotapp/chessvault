use std::sync::LazyLock;

use chess_core::{Piece, Position, Side, Square};
use iced::widget::{Space, column, container, responsive, row, stack, svg, text};
use iced::{Color, Element, Fill};

const LIGHT: Color = Color::from_rgb8(230, 230, 230);
const DARK: Color = Color::from_rgb8(26, 26, 26);

// Embed assets so rendering is independent of the working directory. Reuse
// handles across redraws to keep the SVG renderer's cache effective.
static PIECES: LazyLock<[[svg::Handle; 6]; 2]> = LazyLock::new(|| {
    [
        [
            include_bytes!("../assets/rhosgfx-outline/wP.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/wN.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/wB.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/wR.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/wQ.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/wK.svg").as_slice(),
        ],
        [
            include_bytes!("../assets/rhosgfx-outline/bP.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/bN.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/bB.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/bR.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/bQ.svg").as_slice(),
            include_bytes!("../assets/rhosgfx-outline/bK.svg").as_slice(),
        ],
    ]
    .map(|pieces| pieces.map(svg::Handle::from_memory))
});

fn piece_handle(piece: Piece, side: Side) -> Option<svg::Handle> {
    PIECES.get(side as usize)?.get(piece as usize).cloned()
}

pub fn view<'a, Message: 'a>(position: &'a Position) -> Element<'a, Message> {
    responsive(move |size| {
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
                let square = Square::try_from((rank * 8 + file) as u8).expect("board coordinates are valid squares");
                let mut layers = stack![];
                if let Some(handle) = position
                    .piece_at(square)
                    .and_then(|(piece, side)| piece_handle(piece, side))
                {
                    layers = layers.push(
                        container(svg(handle).width(square_size * 0.8).height(square_size * 0.8))
                            .center_x(Fill)
                            .center_y(Fill),
                    );
                }
                layers = layers.push(container(labels).padding(padding).width(Fill).height(Fill));
                squares = squares.push(
                    container(layers)
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
