// If you're looking for zsh, those are maintained by hand in `complete/_rg`.
// The main reason for this is that the zsh completions were:
//
// 1. Lovingly written by an expert in such things.
// 2. Are much higher in quality than the ones below that are auto-generated.
// Namely, the zsh completions take application level context about flag
// compatibility into account.
// 3. There is a CI script that fails if a new flag is added to ripgrep that
// isn't included in the zsh completions.
// 4. There is a wealth of documentation in the zsh script explaining how it
// works and how it can be extended.
//
// In principle, I'd be open to maintaining any completion script by hand so
// long as it meets criteria 3 and 4 above.

pub(super) mod bash;
pub(super) mod fish;
pub(super) mod powershell;
