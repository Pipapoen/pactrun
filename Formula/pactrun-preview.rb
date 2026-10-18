class PactrunPreview < Formula
  desc 'Pactrun immutable revisions and managed instances'
  homepage 'https://github.com/Pipapoen/pactrun'
  license 'MIT'
  url 'https://github.com/Pipapoen/pactrun/releases/download/v1.0.0-alpha.10/pactrun-1.0.0-alpha.10-linux-x86_64.tar.gz'
  version '1.0.0-alpha.10'
  sha256 'f192cbbf5e6c3534b13d76719a33c16e5616855477b0ba526d1b0be77b595eb4'
  depends_on :linux
  depends_on arch: :x86_64
  def install
    Formula.installed.each do |other|
      if other.name != name && (other.opt_bin/"pactrun").exist?
        raise "Uninstall #{other.full_name} before installing #{full_name}; managed data is retained."
      end
    end
    prefix.install 'bin', 'LICENSE', 'THIRD_PARTY_NOTICES.txt', 'rust-licenses'
  end
  test do
    assert_match version.to_s, shell_output("#{bin}/pactrun --version")
  end
end
