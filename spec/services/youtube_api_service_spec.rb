require "rails_helper"

RSpec.describe YoutubeAPIService do
  let(:api_key) { "test_api_key" }
  let(:service) { described_class.new(api_key: api_key) }

  describe "#initialize" do
    it "raises an error when API key is blank" do
      expect { described_class.new(api_key: nil) }.to raise_error(YoutubeAPIService::Error, "YouTube API key not configured")
    end
  end

  describe "#fetch_playlist_info" do
    it "returns playlist information" do
      stub_request(:get, "https://www.googleapis.com/youtube/v3/playlists")
        .with(query: { part: "snippet,contentDetails", id: "PLtest", key: api_key })
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: {
            items: [{
              snippet: {
                title: "My Playlist",
                channelTitle: "Test Channel",
                thumbnails: { high: { url: "https://i.ytimg.com/vi/abc/hqdefault.jpg" } }
              },
              contentDetails: { itemCount: 10 }
            }]
          }.to_json
        )

      result = service.fetch_playlist_info("PLtest")

      expect(result[:title]).to eq("My Playlist")
      expect(result[:channel_name]).to eq("Test Channel")
      expect(result[:thumbnail_url]).to eq("https://i.ytimg.com/vi/abc/hqdefault.jpg")
      expect(result[:item_count]).to eq(10)
    end

    it "raises error when playlist is not found" do
      stub_request(:get, "https://www.googleapis.com/youtube/v3/playlists")
        .with(query: { part: "snippet,contentDetails", id: "PLnotfound", key: api_key })
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: { items: [] }.to_json
        )

      expect { service.fetch_playlist_info("PLnotfound") }.to raise_error(YoutubeAPIService::Error, "Playlist not found")
    end

    it "raises error on API failure" do
      stub_request(:get, "https://www.googleapis.com/youtube/v3/playlists")
        .with(query: { part: "snippet,contentDetails", id: "PLtest", key: api_key })
        .to_return(
          status: 403,
          headers: { "Content-Type" => "application/json" },
          body: { error: { message: "Forbidden" } }.to_json
        )

      expect { service.fetch_playlist_info("PLtest") }.to raise_error(YoutubeAPIService::Error, "Forbidden")
    end
  end

  describe "#fetch_playlist_items" do
    it "returns playlist items across pages" do
      stub_request(:get, "https://www.googleapis.com/youtube/v3/playlistItems")
        .with(query: { part: "snippet", playlistId: "PLtest", maxResults: "50", key: api_key })
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: {
            items: [{
              snippet: {
                resourceId: { videoId: "vid1" },
                title: "Song 1",
                position: 0,
                thumbnails: { default: { url: "https://example.com/thumb1.jpg" } }
              }
            }],
            nextPageToken: "page2"
          }.to_json
        )

      stub_request(:get, "https://www.googleapis.com/youtube/v3/playlistItems")
        .with(query: { part: "snippet", playlistId: "PLtest", maxResults: "50", pageToken: "page2", key: api_key })
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: {
            items: [{
              snippet: {
                resourceId: { videoId: "vid2" },
                title: "Song 2",
                position: 1,
                thumbnails: { default: { url: "https://example.com/thumb2.jpg" } }
              }
            }]
          }.to_json
        )

      items = service.fetch_playlist_items("PLtest")

      expect(items.length).to eq(2)
      expect(items[0][:video_id]).to eq("vid1")
      expect(items[0][:title]).to eq("Song 1")
      expect(items[1][:video_id]).to eq("vid2")
    end
  end

  describe "#fetch_video_details" do
    it "returns video details with parsed duration" do
      stub_request(:get, "https://www.googleapis.com/youtube/v3/videos")
        .with(query: { part: "snippet,contentDetails", id: "vid1,vid2", key: api_key })
        .to_return(
          status: 200,
          headers: { "Content-Type" => "application/json" },
          body: {
            items: [
              {
                id: "vid1",
                snippet: { title: "Song 1", channelTitle: "Artist 1" },
                contentDetails: { duration: "PT3M45S" }
              },
              {
                id: "vid2",
                snippet: { title: "Song 2", channelTitle: "Artist 2" },
                contentDetails: { duration: "PT1H2M30S" }
              }
            ]
          }.to_json
        )

      details = service.fetch_video_details(%w[vid1 vid2])

      expect(details.length).to eq(2)
      expect(details[0][:video_id]).to eq("vid1")
      expect(details[0][:duration]).to eq(225.0) # 3*60 + 45
      expect(details[1][:duration]).to eq(3750.0) # 1*3600 + 2*60 + 30
    end

    it "returns empty array for empty input" do
      expect(service.fetch_video_details([])).to eq([])
    end
  end
end
