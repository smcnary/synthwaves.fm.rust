require "rails_helper"

RSpec.describe ThemeHelper, type: :helper do
  describe "#current_theme" do
    it "returns the user's theme when logged in" do
      user = create(:user, theme: "jazz")
      allow(Current).to receive(:user).and_return(user)
      expect(helper.current_theme).to eq("jazz")
    end

    it "returns default when no user is logged in" do
      allow(Current).to receive(:user).and_return(nil)
      expect(helper.current_theme).to eq("synthwave")
    end

    it "returns default for invalid theme values" do
      user = create(:user)
      allow(user).to receive(:theme).and_return("vaporwave")
      allow(Current).to receive(:user).and_return(user)
      expect(helper.current_theme).to eq("synthwave")
    end
  end

  describe "#current_theme_config" do
    it "returns the config hash for the current theme" do
      user = create(:user, theme: "reggae")
      allow(Current).to receive(:user).and_return(user)
      config = helper.current_theme_config
      expect(config[:label]).to eq("Reggae")
      expect(config[:font_family]).to eq("Righteous")
    end
  end

  describe "#current_theme_font_url" do
    it "returns the Google Fonts URL for the current theme" do
      user = create(:user, theme: "punk")
      allow(Current).to receive(:user).and_return(user)
      expect(helper.current_theme_font_url).to include("Rubik+Mono+One")
    end
  end

  describe "#current_theme_meta_color" do
    it "returns the meta color for the current theme" do
      user = create(:user, theme: "jazz")
      allow(Current).to receive(:user).and_return(user)
      expect(helper.current_theme_meta_color).to eq("#0d0f1a")
    end
  end

  describe "#theme_fonts_json" do
    it "returns a JSON map of all theme font URLs" do
      parsed = JSON.parse(helper.theme_fonts_json)
      expect(parsed.keys).to match_array(Themeable::THEMES.keys)
      expect(parsed["synthwave"]).to include("Orbitron")
    end
  end
end
