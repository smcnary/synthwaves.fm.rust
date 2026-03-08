class TrackResource < Madmin::Resource
  attribute :id, form: false
  attribute :title
  attribute :artist
  attribute :album
  attribute :track_number
  attribute :duration
  attribute :file_format
  attribute :disc_number, index: false
  attribute :file_size, index: false
  attribute :bitrate, index: false
  attribute :youtube_video_id, index: false
  attribute :audio_file, index: false
  attribute :playlist_tracks, index: false
  attribute :playlists, index: false
  attribute :play_histories, index: false
  attribute :favorites, index: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.title
  def self.default_sort_column = "title"
end
