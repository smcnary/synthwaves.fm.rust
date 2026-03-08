require "rails_helper"

RSpec.describe RadioStation, type: :model do
  describe "associations" do
    it { is_expected.to belong_to(:user) }
  end

  describe "validations" do
    it { is_expected.to validate_presence_of(:name) }
    it { is_expected.to validate_presence_of(:youtube_url) }

    it "rejects a playlist URL with a helpful message" do
      station = build(:radio_station, youtube_url: "https://music.youtube.com/playlist?list=OLAK5uy_abc", youtube_video_id: nil)
      station.valid?
      expect(station.errors[:youtube_url].first).to include("playlist URL")
      expect(station.errors[:youtube_url].first).to include("Import from YouTube")
    end

    it "rejects a non-YouTube URL" do
      station = build(:radio_station, youtube_url: "https://example.com", youtube_video_id: nil)
      station.valid?
      expect(station.errors[:youtube_url].first).to include("doesn't appear to be a valid")
    end
  end

  describe "before_validation" do
    it "extracts video ID from youtube_url when youtube_video_id is blank" do
      station = build(:radio_station, youtube_url: "https://www.youtube.com/watch?v=jfKfPfyJRdk", youtube_video_id: nil)
      station.valid?
      expect(station.youtube_video_id).to eq("jfKfPfyJRdk")
    end

    it "does not overwrite an existing youtube_video_id" do
      station = build(:radio_station, youtube_url: "https://www.youtube.com/watch?v=jfKfPfyJRdk", youtube_video_id: "existing123")
      station.valid?
      expect(station.youtube_video_id).to eq("existing123")
    end
  end
end
