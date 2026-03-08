require "rails_helper"

RSpec.describe YoutubeUrlParser do
  describe ".extract_video_id" do
    it "extracts from standard youtube.com/watch URL" do
      expect(described_class.extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ")).to eq("dQw4w9WgXcQ")
    end

    it "extracts from youtu.be short URL" do
      expect(described_class.extract_video_id("https://youtu.be/dQw4w9WgXcQ")).to eq("dQw4w9WgXcQ")
    end

    it "extracts from youtube.com/live URL" do
      expect(described_class.extract_video_id("https://www.youtube.com/live/jfKfPfyJRdk")).to eq("jfKfPfyJRdk")
    end

    it "extracts from embed URL" do
      expect(described_class.extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ")).to eq("dQw4w9WgXcQ")
    end

    it "extracts from URL with extra parameters" do
      expect(described_class.extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf")).to eq("dQw4w9WgXcQ")
    end

    it "returns nil for blank input" do
      expect(described_class.extract_video_id("")).to be_nil
      expect(described_class.extract_video_id(nil)).to be_nil
    end

    it "returns nil for non-YouTube URLs" do
      expect(described_class.extract_video_id("https://vimeo.com/12345")).to be_nil
    end
  end

  describe ".extract_playlist_id" do
    it "extracts from youtube.com/playlist URL" do
      expect(described_class.extract_playlist_id("https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf")).to eq("PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf")
    end

    it "extracts from music.youtube.com/playlist URL" do
      expect(described_class.extract_playlist_id("https://music.youtube.com/playlist?list=OLAK5uy_abc123")).to eq("OLAK5uy_abc123")
    end

    it "extracts from watch URL with list parameter" do
      expect(described_class.extract_playlist_id("https://www.youtube.com/watch?v=abc&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf")).to eq("PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf")
    end

    it "returns nil for blank input" do
      expect(described_class.extract_playlist_id("")).to be_nil
      expect(described_class.extract_playlist_id(nil)).to be_nil
    end

    it "returns nil for URLs without playlist" do
      expect(described_class.extract_playlist_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ")).to be_nil
    end
  end

  describe ".video_url?" do
    it "returns true for video URLs" do
      expect(described_class.video_url?("https://youtu.be/dQw4w9WgXcQ")).to be true
    end

    it "returns false for non-video URLs" do
      expect(described_class.video_url?("https://example.com")).to be false
    end
  end

  describe ".playlist_url?" do
    it "returns true for playlist URLs" do
      expect(described_class.playlist_url?("https://www.youtube.com/playlist?list=PLabc")).to be true
    end

    it "returns false for non-playlist URLs" do
      expect(described_class.playlist_url?("https://www.youtube.com/watch?v=abc")).to be false
    end
  end
end
