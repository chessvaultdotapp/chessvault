# Games, history, and results

Use `Game::play` rather than changing a separate position when maintaining a game:
`Game` records moves and repetition history and rejects moves after a result.
`position()` exposes the live position read-only. `position_at(ply)` reconstructs
an independent position without rewinding the game; zero selects the starting position.

`PositionStatus` classifies ongoing play, checkmate, stalemate, and proven dead
material. `GameStatus` also accounts for resignation, agreement, and history-based
draws. `Checkmate(side)` and `Resigned(side)` identify the loser, not the winner.

## Draw claims and automatic results

Threefold repetition and the fifty-move rule require a claim. Fivefold repetition
and the seventy-five-move rule are automatic. Intended-move claims leave the board
and recorded history unchanged. Dead-position detection is conservative: bare
kings, a single minor piece, and bishops all on one colour are detected; blocked
dead positions with pawns are not.

## Draw offers

Draw offers are made by the side to move and survive that player's move. Only
the opponent can accept or decline; the opponent's next legal move implicitly
declines. Invalid moves preserve the offer, and a game result clears it.

See [Positions and moves](positions.md) for move encoding and board-based outcomes.
