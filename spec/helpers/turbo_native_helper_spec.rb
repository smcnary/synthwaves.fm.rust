require "rails_helper"

RSpec.describe TurboNativeHelper, type: :helper do
  describe "#turbo_native_app?" do
    it "returns true when User-Agent contains 'Turbo Native'" do
      allow(helper.request).to receive(:user_agent).and_return("SynthwavesFM/1.0 Turbo Native iOS")
      expect(helper.turbo_native_app?).to be true
    end

    it "returns false for a standard browser User-Agent" do
      allow(helper.request).to receive(:user_agent).and_return("Mozilla/5.0 (Macintosh)")
      expect(helper.turbo_native_app?).to be false
    end

    it "returns false when User-Agent is nil" do
      allow(helper.request).to receive(:user_agent).and_return(nil)
      expect(helper.turbo_native_app?).to be false
    end
  end
end
