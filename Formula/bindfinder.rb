class Bindfinder < Formula
  desc "Terminal-first command reference browser for SSH, tmux, and shell-heavy workflows"
  homepage "https://github.com/younesehb/bindfinder"
  license "MIT"
  head "https://github.com/younesehb/bindfinder.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
    man1.install "man/bindfinder.1"
  end

  test do
    assert_match "bindfinder", shell_output("#{bin}/bindfinder --help")
  end
end
