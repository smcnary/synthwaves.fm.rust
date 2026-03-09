require "rails_helper"

RSpec.describe "Videos", type: :request do
  let(:user) { create(:user) }

  before { login_user(user) }

  describe "GET /videos/new" do
    it "returns success" do
      get new_video_path
      expect(response).to have_http_status(:ok)
    end
  end

  describe "POST /videos" do
    it "creates a video with an uploaded file" do
      file = fixture_file_upload("test.mp4", "video/mp4")

      expect {
        post videos_path, params: {video_file: file, title: "My Video"}
      }.to change(Video, :count).by(1)

      video = Video.last
      expect(video.title).to eq("My Video")
      expect(video.status).to eq("processing")
      expect(video.user).to eq(user)
      expect(response).to redirect_to(video_path(video))
    end

    it "uses filename as title when title is blank" do
      file = fixture_file_upload("test.mp4", "video/mp4")

      post videos_path, params: {video_file: file}

      expect(Video.last.title).to eq("test")
    end

    it "enqueues VideoConversionJob" do
      file = fixture_file_upload("test.mp4", "video/mp4")

      expect {
        post videos_path, params: {video_file: file, title: "Test"}
      }.to have_enqueued_job(VideoConversionJob)
    end

    it "rejects when no file is attached" do
      post videos_path, params: {title: "No File"}
      expect(response).to have_http_status(:unprocessable_content)
    end
  end

  describe "GET /videos/:id" do
    it "returns success for own video" do
      video = create(:video, user: user)
      get video_path(video)
      expect(response).to have_http_status(:ok)
    end

    it "shows processing state" do
      video = create(:video, :processing, user: user)
      get video_path(video)
      expect(response.body).to include("Processing video")
    end

    it "shows failed state" do
      video = create(:video, :failed, user: user)
      get video_path(video)
      expect(response.body).to include("Processing failed")
    end

    it "returns 404 for another user's video" do
      other_user = create(:user)
      video = create(:video, user: other_user)

      get video_path(video)
      expect(response).to have_http_status(:not_found)
    end
  end

  describe "GET /videos/:id/edit" do
    it "returns success" do
      video = create(:video, user: user)
      get edit_video_path(video)
      expect(response).to have_http_status(:ok)
    end
  end

  describe "PATCH /videos/:id" do
    it "updates the video" do
      video = create(:video, user: user, title: "Old Title")
      patch video_path(video), params: {video: {title: "New Title", description: "Updated"}}

      video.reload
      expect(video.title).to eq("New Title")
      expect(video.description).to eq("Updated")
      expect(response).to redirect_to(video_path(video))
    end

    it "rejects blank title" do
      video = create(:video, user: user)
      patch video_path(video), params: {video: {title: ""}}
      expect(response).to have_http_status(:unprocessable_content)
    end
  end

  describe "DELETE /videos/:id" do
    it "deletes the video" do
      video = create(:video, user: user)
      expect { delete video_path(video) }.to change(Video, :count).by(-1)
      expect(response).to redirect_to(tv_path(tab: "videos"))
    end
  end

  describe "GET /videos/:id/stream" do
    it "redirects to blob url for ready videos" do
      video = create(:video, user: user, status: "ready")
      video.file.attach(io: StringIO.new("fake video"), filename: "test.mp4", content_type: "video/mp4")

      get stream_video_path(video)
      expect(response).to have_http_status(:redirect)
    end

    it "returns 404 for processing videos" do
      video = create(:video, :processing, user: user)
      video.file.attach(io: StringIO.new("fake video"), filename: "test.mp4", content_type: "video/mp4")

      get stream_video_path(video)
      expect(response).to have_http_status(:not_found)
    end

    it "returns 404 for videos without attached file" do
      video = create(:video, user: user, status: "ready")

      get stream_video_path(video)
      expect(response).to have_http_status(:not_found)
    end
  end
end
