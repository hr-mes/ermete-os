# Ermete Forge - Local Build

`build_all_local.sh` was removed because it represented a monolithic hack violating the micro-container OCI principle.

If you need to build packages locally, follow the rigid dependency DAG implicitly defined in our GitHub Actions:
1. `appmenu-glib-translator`
2. `astal-io`
3. `astal`
4. `astal-libs`
5. `astal-gjs`
6. `astal-gtk4`
7. `astal-lua`
8. `aylurs-gtk-shell2`
9. `hyprpanel`

Every package must be built individually, installed in the local environment, and then the next package can be built.
