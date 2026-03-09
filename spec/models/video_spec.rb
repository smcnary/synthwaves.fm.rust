require "rails_helper"

RSpec.describe Video, type: :model do
  describe "associations" do
    it { should belong_to(:user) }
    it { should have_one_attached(:file) }
    it { should have_one_attached(:thumbnail) }
    it { should have_many(:favorites).dependent(:destroy) }
  end

  describe "validations" do
    it { should validate_presence_of(:title) }
  end

  describe ".ready" do
    it "returns only videos with ready status" do
      ready_video = create(:video, status: "ready")
      processing_video = create(:video, :processing)
      failed_video = create(:video, :failed)

      expect(Video.ready).to include(ready_video)
      expect(Video.ready).not_to include(processing_video, failed_video)
    end
  end

  describe ".search" do
    it "returns videos matching by title" do
      matching = create(:video, title: "Concert Footage")
      non_matching = create(:video, title: "Tutorial")

      results = Video.search("Concert")

      expect(results).to include(matching)
      expect(results).not_to include(non_matching)
    end

    it "returns all videos when query is blank" do
      videos = create_list(:video, 3)
      expect(Video.search("")).to match_array(videos)
    end

    it "returns all videos when query is nil" do
      videos = create_list(:video, 3)
      expect(Video.search(nil)).to match_array(videos)
    end
  end

  describe "callbacks" do
    it "enqueues VideoConversionJob after create" do
      video = build(:video, status: "processing")
      video.file.attach(
        io: StringIO.new("fake video"),
        filename: "test.mp4",
        content_type: "video/mp4"
      )

      expect { video.save! }.to have_enqueued_job(VideoConversionJob).with(video.id)
    end
  end
end
