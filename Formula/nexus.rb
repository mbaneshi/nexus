class Nexus < Formula
  desc "Home command center: discovery, config management, and AI — one binary, complete picture of your machine"
  homepage "https://github.com/mbaneshi/nexus"
  head "https://github.com/mbaneshi/nexus.git", branch: "main"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/cli")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/nexus --version")
  end
end
