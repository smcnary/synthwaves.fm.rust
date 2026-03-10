class MediaDownloadJob < ApplicationJob
  queue_as :default

  def perform(track_id, url, user_id:)
    track = Track.find(track_id)
    return if track.audio_file.attached?

    temp_dir = Rails.root.join("tmp/media_downloads/track_#{track_id}_#{SecureRandom.hex(4)}")
    FileUtils.mkdir_p(temp_dir)

    track.update!(download_status: "downloading", download_error: nil)
    broadcast_status(track, user_id)

    file_path = MediaDownloadService.download_audio(url, output_dir: temp_dir.to_s)

    track.audio_file.attach(
      io: File.open(file_path),
      filename: "#{track_id}.mp3",
      content_type: "audio/mpeg"
    )

    metadata = begin
      MetadataExtractor.call(file_path)
    rescue WahWah::WahWahArgumentError
      {}
    end

    track.update!(
      download_status: "completed",
      download_error: nil,
      duration: metadata[:duration] || track.duration,
      bitrate: metadata[:bitrate] || track.bitrate,
      file_format: "mp3",
      file_size: File.size(file_path)
    )
    broadcast_status(track, user_id)
  rescue MediaDownloadService::Error, StandardError => e
    track&.update!(download_status: "failed", download_error: e.message.truncate(500))
    broadcast_status(track, user_id) if track
    raise unless e.is_a?(MediaDownloadService::Error)
  ensure
    FileUtils.rm_rf(temp_dir) if temp_dir
  end

  private

  def broadcast_status(track, user_id)
    return unless track

    Turbo::StreamsChannel.broadcast_replace_to(
      "downloads_#{user_id}",
      target: "media-download-track-#{track.id}",
      partial: "youtube_imports/download_status",
      locals: { record: track, type: "track" }
    )
  end
end
