use anyhow::Result;

fn main() -> Result<()> {
    cka_buddy_tui::app::run_with_args(std::env::args().skip(1).collect())
}
