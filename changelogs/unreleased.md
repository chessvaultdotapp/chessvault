# Unreleased

## Added

- Added a **Claim draw** button for threefold repetition and the fifty-move rule. Qualifying intended moves also
  appear as draw-claim buttons, allowing a claim before playing the move without changing the board or move history.
  The fifty-move clock resets after any pawn move or capture.

- Added automatic draws for fivefold repetition and the seventy-five-move rule. Checkmate on the final move takes
  precedence over a seventy-five-move draw.

- Added automatic dead-position draws for bare kings, king and one bishop or knight against a bare king, and
  bishop-only material where all bishops occupy the same square colour. Other dead positions, such as blocked-pawn
  positions, are not yet detected. Two knights and opposite-coloured bishops are not automatically treated as dead.

- Added draw offers with opponent accept and decline controls. An offer survives the offering player's move;
  the opponent's next legal move automatically declines it. Accepted offers end the game by agreement without
  changing the board or move history. Draw actions are unavailable during history review or modal interactions.

## Changed

- The **New game** button is now available after a game ends even if no moves have been played.
