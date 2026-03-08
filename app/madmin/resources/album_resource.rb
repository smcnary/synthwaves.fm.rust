class AlbumResource < Madmin::Resource
  attribute :id, form: false
  attribute :title
  attribute :artist
  attribute :year
  attribute :genre
  attribute :youtube_playlist_url, index: false
  attribute :cover_image, index: false
  attribute :tracks, index: false
  attribute :favorites, index: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.title
  def self.default_sort_column = "title"
end
