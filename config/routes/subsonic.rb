# Subsonic API
namespace :api do
  namespace :subsonic, path: "/rest" do
    get "ping.view", to: "system#ping"
    post "ping.view", to: "system#ping"
    get "getLicense.view", to: "system#get_license"
    post "getLicense.view", to: "system#get_license"

    get "getMusicFolders.view", to: "browsing#get_music_folders"
    post "getMusicFolders.view", to: "browsing#get_music_folders"
    get "getIndexes.view", to: "browsing#get_indexes"
    post "getIndexes.view", to: "browsing#get_indexes"
    get "getArtists.view", to: "browsing#get_artists"
    post "getArtists.view", to: "browsing#get_artists"
    get "getArtist.view", to: "browsing#get_artist"
    post "getArtist.view", to: "browsing#get_artist"
    get "getAlbum.view", to: "browsing#get_album"
    post "getAlbum.view", to: "browsing#get_album"
    get "getSong.view", to: "browsing#get_song"
    post "getSong.view", to: "browsing#get_song"

    get "stream.view", to: "media#stream"
    post "stream.view", to: "media#stream"
    get "getCoverArt.view", to: "media#get_cover_art"
    post "getCoverArt.view", to: "media#get_cover_art"

    get "search3.view", to: "search#search3"
    post "search3.view", to: "search#search3"

    get "getAlbumList2.view", to: "lists#get_album_list2"
    post "getAlbumList2.view", to: "lists#get_album_list2"
    get "getRandomSongs.view", to: "lists#get_random_songs"
    post "getRandomSongs.view", to: "lists#get_random_songs"

    get "getPlaylists.view", to: "playlists#get_playlists"
    post "getPlaylists.view", to: "playlists#get_playlists"
    get "getPlaylist.view", to: "playlists#get_playlist"
    post "getPlaylist.view", to: "playlists#get_playlist"
    get "createPlaylist.view", to: "playlists#create_playlist"
    post "createPlaylist.view", to: "playlists#create_playlist"
    get "deletePlaylist.view", to: "playlists#delete_playlist"
    post "deletePlaylist.view", to: "playlists#delete_playlist"

    get "star.view", to: "interaction#star"
    post "star.view", to: "interaction#star"
    get "unstar.view", to: "interaction#unstar"
    post "unstar.view", to: "interaction#unstar"
    get "scrobble.view", to: "interaction#scrobble"
    post "scrobble.view", to: "interaction#scrobble"
  end
end
