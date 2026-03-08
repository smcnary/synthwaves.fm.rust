class CoverArtAttachJob < ApplicationJob
  retry_on StandardError, wait: 5.seconds, attempts: 3

  def perform(album, cover_art_data, mime_type)
    return if album.cover_image.attached?

    album.cover_image.attach(
      io: StringIO.new(Base64.strict_decode64(cover_art_data)),
      filename: "cover.jpg",
      content_type: mime_type || "image/jpeg"
    )
  end
end
