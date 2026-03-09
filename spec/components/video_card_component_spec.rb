require "rails_helper"

RSpec.describe VideoCardComponent, type: :component do
  include ViewComponent::TestHelpers

  let(:video) { create(:video, title: "My Video", duration: 125.0, height: 1080) }

  def render_component(video:)
    render_inline(described_class.new(video: video))
  end

  it "renders the video title" do
    html = render_component(video: video)
    expect(html.text).to include("My Video")
  end

  it "renders duration badge" do
    html = render_component(video: video)
    expect(html.text).to include("2:05")
  end

  it "renders resolution label" do
    html = render_component(video: video)
    expect(html.text).to include("1080p")
  end

  it "shows processing indicator" do
    video = create(:video, :processing, title: "Processing Video")
    html = render_component(video: video)
    expect(html.at_css(".animate-spin")).to be_present
  end

  it "shows failed indicator" do
    video = create(:video, :failed, title: "Failed Video")
    html = render_component(video: video)
    expect(html.text).to include("Failed")
  end

  it "links to video show page" do
    html = render_component(video: video)
    link = html.at_css("a")
    expect(link["href"]).to eq("/videos/#{video.id}")
  end
end
