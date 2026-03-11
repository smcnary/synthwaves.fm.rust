class AlbumMergeService
  class Error < StandardError; end

  def self.call(target:, source:)
    new(target: target, source: source).call
  end

  def initialize(target:, source:)
    @target = target
    @source = source
  end

  def call
    raise Error, "Cannot merge an album into itself." if @target.id == @source.id

    ActiveRecord::Base.transaction do
      @source.tracks.find_each do |track|
        track.update!(album: @target, artist: @target.artist)
      end

      if !@target.cover_image.attached? && @source.cover_image.attached?
        @target.cover_image.attach(@source.cover_image.blob)
      end

      @source.destroy!
    end
  end
end
