class YoutubeImportJob < ApplicationJob
  queue_as :default

  def perform(url, category: "music")
    YoutubePlaylistImportService.call(url, category: category)
  end
end
