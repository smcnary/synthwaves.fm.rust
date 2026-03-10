require "rails_helper"

RSpec.describe "Taggings", type: :request do
  let(:user) { create(:user) }
  let(:track) { create(:track) }

  before { login_user(user) }

  describe "POST /taggings" do
    it "creates a new tag and tagging" do
      expect {
        post taggings_path, params: {tagging: {name: "Synthwave", tag_type: "genre", taggable_type: "Track", taggable_id: track.id}}
      }.to change(Tagging, :count).by(1).and change(Tag, :count).by(1)
    end

    it "reuses an existing tag" do
      create(:tag, name: "synthwave", tag_type: "genre")

      expect {
        post taggings_path, params: {tagging: {name: "Synthwave", tag_type: "genre", taggable_type: "Track", taggable_id: track.id}}
      }.to change(Tagging, :count).by(1).and change(Tag, :count).by(0)
    end

    it "returns turbo_stream response" do
      post taggings_path, params: {tagging: {name: "chill", tag_type: "mood", taggable_type: "Track", taggable_id: track.id}}, as: :turbo_stream
      expect(response).to have_http_status(:ok)
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end

    it "prevents duplicate taggings" do
      tag = create(:tag, name: "rock", tag_type: "genre")
      create(:tagging, tag: tag, taggable: track, user: user)

      expect {
        post taggings_path, params: {tagging: {name: "Rock", tag_type: "genre", taggable_type: "Track", taggable_id: track.id}}
      }.not_to change(Tagging, :count)
    end
  end

  describe "DELETE /taggings/:id" do
    it "removes the tagging" do
      tag = create(:tag, name: "jazz", tag_type: "genre")
      tagging = create(:tagging, tag: tag, taggable: track, user: user)

      expect {
        delete tagging_path(tagging)
      }.to change(Tagging, :count).by(-1)
    end

    it "does not allow deleting another user's tagging" do
      other_user = create(:user)
      tag = create(:tag, name: "jazz", tag_type: "genre")
      tagging = create(:tagging, tag: tag, taggable: track, user: other_user)

      delete tagging_path(tagging)
      expect(response).to have_http_status(:not_found)
    end

    it "returns turbo_stream response" do
      tag = create(:tag, name: "jazz", tag_type: "genre")
      tagging = create(:tagging, tag: tag, taggable: track, user: user)

      delete tagging_path(tagging), as: :turbo_stream
      expect(response).to have_http_status(:ok)
      expect(response.media_type).to eq("text/vnd.turbo-stream.html")
    end
  end
end
