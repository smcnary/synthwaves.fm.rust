require "rails_helper"

RSpec.describe "YoutubeImports", type: :request do
  let(:user) { create(:user) }

  before do
    login_user(user)
    Flipper.enable(:youtube_import)
    allow(Rails.application.credentials).to receive(:youtube_api_key).and_return("test_key")
  end

  describe "GET /youtube_imports/new" do
    it "returns success" do
      get new_youtube_import_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /youtube_imports" do
    it "enqueues an import job and redirects to library" do
      post youtube_imports_path, params: {youtube_url: "https://www.youtube.com/playlist?list=PLtest123"}

      expect(YoutubeImportJob).to have_been_enqueued.with("https://www.youtube.com/playlist?list=PLtest123", category: "music")
      expect(response).to redirect_to(library_path)
      follow_redirect!
      expect(response.body).to include("Playlist import started")
    end

    it "passes the category parameter to the job" do
      post youtube_imports_path, params: {
        youtube_url: "https://www.youtube.com/playlist?list=PLtest123",
        category: "podcast"
      }

      expect(YoutubeImportJob).to have_been_enqueued.with("https://www.youtube.com/playlist?list=PLtest123", category: "podcast")
    end

    it "rejects invalid URLs without enqueuing a job" do
      post youtube_imports_path, params: {youtube_url: "https://example.com"}

      expect(YoutubeImportJob).not_to have_been_enqueued
      expect(response).to have_http_status(:unprocessable_content)
    end

    context "with a single video URL" do
      let(:video_url) { "https://youtu.be/R-FxmoVM7X4" }

      it "imports synchronously and redirects to the album" do
        album = create(:album)
        track = instance_double(Track, album: album)
        allow(YoutubeVideoImportService).to receive(:call).and_return(track)

        post youtube_imports_path, params: {youtube_url: video_url}

        expect(YoutubeVideoImportService).to have_received(:call).with(video_url, category: "music")
        expect(YoutubeImportJob).not_to have_been_enqueued
        expect(response).to redirect_to(album_path(album))
      end

      it "passes the category parameter to the service" do
        album = create(:album)
        track = instance_double(Track, album: album)
        allow(YoutubeVideoImportService).to receive(:call).and_return(track)

        post youtube_imports_path, params: {youtube_url: video_url, category: "podcast"}

        expect(YoutubeVideoImportService).to have_received(:call).with(video_url, category: "podcast")
      end

      it "renders the form with an error when the service fails" do
        allow(YoutubeVideoImportService).to receive(:call)
          .and_raise(YoutubeVideoImportService::Error, "Video not found")

        post youtube_imports_path, params: {youtube_url: video_url}

        expect(response).to have_http_status(:unprocessable_content)
      end
    end

    context "with a URL containing both video and playlist params" do
      it "treats it as a playlist import" do
        url = "https://www.youtube.com/watch?v=R-FxmoVM7X4&list=PLtest123"

        post youtube_imports_path, params: {youtube_url: url}

        expect(YoutubeImportJob).to have_been_enqueued.with(url, category: "music")
      end
    end
  end
end
