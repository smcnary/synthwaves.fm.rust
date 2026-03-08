class SearchService
  def self.call(query:, types: [:artist, :album, :track], limit: 20,
    genre: nil, year_from: nil, year_to: nil,
    favorites_only: false, user: nil)
    new(query: query, types: types, limit: limit,
      genre: genre, year_from: year_from, year_to: year_to,
      favorites_only: favorites_only, user: user).call
  end

  def initialize(query:, types:, limit:, genre: nil, year_from: nil, year_to: nil,
    favorites_only: false, user: nil)
    @query = query
    @types = types
    @limit = limit
    @genre = genre
    @year_from = year_from
    @year_to = year_to
    @favorites_only = favorites_only
    @user = user
  end

  def call
    pattern = "%#{@query}%"
    {
      artists: search_artists(pattern),
      albums: search_albums(pattern),
      tracks: search_tracks(pattern)
    }
  end

  private

  def search_artists(pattern)
    return [] unless @types.include?(:artist)
    scope = Artist.where("name LIKE ?", pattern)
    scope = scope.where(id: @user.favorites.where(favorable_type: "Artist").select(:favorable_id)) if @favorites_only && @user
    scope.limit(@limit)
  end

  def search_albums(pattern)
    return [] unless @types.include?(:album)
    scope = Album.includes(:artist).where("title LIKE ?", pattern)
    scope = scope.where(genre: @genre) if @genre.present?
    scope = scope.where("year >= ?", @year_from) if @year_from
    scope = scope.where("year <= ?", @year_to) if @year_to
    scope = scope.where(id: @user.favorites.where(favorable_type: "Album").select(:favorable_id)) if @favorites_only && @user
    scope.limit(@limit)
  end

  def search_tracks(pattern)
    return [] unless @types.include?(:track)
    scope = Track.includes(:artist, :album).where("tracks.title LIKE ?", pattern)
    if @genre.present? || @year_from || @year_to
      scope = scope.joins(:album)
      scope = scope.where(albums: {genre: @genre}) if @genre.present?
      scope = scope.where("albums.year >= ?", @year_from) if @year_from
      scope = scope.where("albums.year <= ?", @year_to) if @year_to
    end
    scope = scope.where(id: @user.favorites.where(favorable_type: "Track").select(:favorable_id)) if @favorites_only && @user
    scope.limit(@limit)
  end
end
