require "rails_helper"

RSpec.describe YoutubeImportJob, type: :job do
  it "calls YoutubePlaylistImportService with the url and category" do
    allow(YoutubePlaylistImportService).to receive(:call)

    described_class.perform_now("https://www.youtube.com/playlist?list=PLtest123", category: "podcast")

    expect(YoutubePlaylistImportService).to have_received(:call)
      .with("https://www.youtube.com/playlist?list=PLtest123", category: "podcast")
  end
end
