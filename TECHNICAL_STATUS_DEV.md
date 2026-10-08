# FreokRO Korangar — development checkpoint — October 7, 2026

This file records the **initial FreokRO `dev` snapshot before any integration**. `vE5li/korangar` tag `v0.1.1-20260220` identifies the original code source, not a FreokRO Git base. At preservation preparation, the FreokRO remote had no branches, including `main`; a real rebase therefore had no base. The local `dev` branch initially had no commits, and the upstream tag remains only a reference. The separate FreokRO rAthena server and private Auction HUD are linked in the README.

## Architecture and implemented work

- **IMPLEMENTED:** protocol adaptation for `PACKETVER 20220406` without packet obfuscation. The Ragexe/EXE `20250716` installation is separate. Episode 19 logic is primarily server-side; its presentation in this client requires game testing.
- **IMPLEMENTED:** item descriptions for inventory and shops in `korangar/src/interface/windows/item_details.rs`, including icon, identification state, description, facts, cards, and random options. The window uses the standard title-bar drag handler in `korangar-interface/src/window/mod.rs`.
- **IMPLEMENTED:** item-description text preserves Ragnarok `^RRGGBB` colors through the existing font renderer and shows underscore separators as horizontal rules. Equipped cards now show their item sprites beside their names when the assets load. These changes still need an in-game visual check.
- **IMPLEMENTED:** the in-game character menu has a **Skin** button backed by `korangar/src/interface/windows/skin.rs`. **Classic** is the first option and default. **Original** retains the previous in-game theme. Existing default-theme settings migrate to Classic; selection persists in `client/interface_settings.ron`.
- **IMPLEMENTED:** Classic applies the auction's dark green palette through `korangar/src/state/theme/interface.rs`. The item-description body received additional matching colors, section bars, and footer styling. Other controls using `InterfaceThemeType::InGame` receive the palette.
- **IMPLEMENTED:** the README explains build and runtime dependencies, cross-repository coupling, and official download links. Game assets, `data-freokro.7z`, saved login settings, and the deployed executable are outside this source tree.

## Build and functional status

- **BUILDS:** `cargo +nightly-2026-02-01 check --locked -p korangar --features unicode,debug` and `cargo +nightly-2026-02-01 build --release --locked -p korangar --features unicode,debug` passed on October 7, 2026. Local `slangc` and NASM were added to `PATH` for the build.
- **WORKING:** after the item-description color and card-icon updates, the same check and release build passed again on October 7, 2026. The new executable was copied to the separate local installation; source and installed SHA256 both equal `1776F9B1EF0DCD105B6F7E1AB08697758B16AD25C85E470387D9A1744FA0E241`. The local in-game theme setting was changed to `Classic`.
- **WORKING:** the user confirmed in a live game session that the item-description window can be dragged. October 7 screenshots show the Apple (ID 512) description open alongside the Auction HUD.
- **PARTIAL:** the shared theme covers controls and windows using `InterfaceThemeType::InGame`; any element with hard-coded colors may still need individual work for complete visual parity with the auction.
- **PARTIAL:** the screenshots show different item text sources: the installed Korangar asset describes Apple in Korean, while the HUD catalog describes it in English. The type/fact rows and window styling also differ. The palettes are closer, but this is not complete item-window or game-interface parity.
- **NOT VALIDATED:** visual inspection of Skin → Classic / Original selection, card icons, all item types, persistence after restart, and the rest of the interface in a real game session after this build. The tool's browser security policy blocked a local HTML preview.
- **NOT VALIDATED:** Episode 19 and all auction paths after the skin change. The user reported that server-side escrow passed in the client after its SQL migration, but this build has not independently repeated that test.
- **PENDING:** compare the actual game UI with the Auction HUD, adjust any components outside the shared theme, and test item descriptions and skin switching.
