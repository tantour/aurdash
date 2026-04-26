# Implementing the Manager Tab & AUR Installed Badge

To implement the features you requested, you will need to add a few things to `aurdash`.

1.  **Track Installed Packages:**
    In `src/app.rs`, add `pub installed_pkgs: std::collections::HashSet<String>` to `App`. Populate this during `App::new()` using `pacman -Qq`.
    ```rust
    // inside App::new()
    let mut installed = std::collections::HashSet::new();
    if let Ok(output) = std::process::Command::new("pacman").arg("-Qq").output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            for line in s.lines() { installed.insert(line.to_string()); }
        }
    }
    ```

2.  **Add `pkg.is_installed()` for AUR:**
    Modify `PkgEntry::is_installed(&self)` in `src/aur/mod.rs` to take the `&App` as an argument, OR just use `app.installed_pkgs.contains(pkg.name())` inside `render_results_list` in `src/ui/search.rs` to render the badge for both repo and AUR packages correctly.

3.  **Create the Manager Tab (`Panel::Manager`):**
    *   Add `Manager` to the `Panel` enum in `src/app.rs`.
    *   Add keybindings for `Tab` in `events.rs` to toggle between `Panel::Search` (or `Panel::Results`) and `Panel::Manager`.
    *   Create a new UI module, e.g., `src/ui/manager.rs`, that reads `app.installed_pkgs` (or does a deeper query `pacman -Qi` or `pacman -Qm` to split them) and renders a `List` widget just like the search results list.

4.  **Manager Actions (Uninstall / Reinstall):**
    *   In `events.rs`, catch `KeyCode::Char('u')` (uninstall) and `KeyCode::Char('r')` (reinstall) when `app.active_panel == Panel::Manager`.
    *   When 'u' is pressed, set a state variable `app.uninstall_popup_open = true`.
    *   Render a small popup (like the comments popup) in `ui/mod.rs` asking: "Remove [N]ormal or [R]ecursive?".
    *   Bind `n` to run `sudo pacman -R <pkg>` and `r` to run `sudo pacman -Rs <pkg>` (or using paru). Wait for the command to finish and refresh the `installed_pkgs` list!

These are the architectural hooks you need! Let me know if you want me to attempt writing any of these exact files out for you.
