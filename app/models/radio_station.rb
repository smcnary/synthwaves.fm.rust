class RadioStation < ApplicationRecord
  belongs_to :user

  validates :name, presence: true
  validates :youtube_url, presence: true
  validate :youtube_url_must_contain_video_id

  before_validation :extract_video_id, if: -> { youtube_url.present? && youtube_video_id.blank? }

  private

  def extract_video_id
    self.youtube_video_id = YoutubeUrlParser.extract_video_id(youtube_url)
  end

  def youtube_url_must_contain_video_id
    return if youtube_url.blank?
    return if youtube_video_id.present?

    if YoutubeUrlParser.playlist_url?(youtube_url)
      errors.add(:youtube_url, "is a playlist URL. Use 'Import from YouTube' to import playlists. This form is for individual video or live stream URLs.")
    else
      errors.add(:youtube_url, "doesn't appear to be a valid YouTube video or live stream URL")
    end
  end
end
