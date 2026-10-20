import type { Game } from "./engine";
import type { Theme } from "./theme";
import type { Command, View } from "./types";

/** What the dialogs of the game screen need. */
export interface Ctx {
  game: Game;
  theme: Theme;
  view: () => View;
  /** Applies a command, shows its error if rejected, refreshes the screen. */
  command: (cmd: Command) => boolean;
}
