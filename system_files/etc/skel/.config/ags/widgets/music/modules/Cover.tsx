import { Gtk } from "ags/gtk4";
import Gio from "gi://Gio?version=2.0";
import Mpris from "gi://AstalMpris";
import { createBinding } from "ags";

export function Cover({ player }: { player: Mpris.Player }) {
  return (
    <Gtk.Picture
      cssClasses={["cover"]}
      contentFit={Gtk.ContentFit.COVER}
      file={createBinding(player, "coverArt").as((path) =>
        Gio.file_new_for_path(path),
      )}
    />
  );
}
