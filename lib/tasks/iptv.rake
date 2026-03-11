namespace :iptv do
  desc "Sync IPTV channels from iptv-org playlist"
  task sync: :environment do
    puts "Syncing IPTV channels..."
    result = IPTVChannelSyncService.call
    puts "Done. Synced #{result[:synced]} channels."
  end

  desc "Sync EPG programme data from tvpass.org"
  task epg_sync: :environment do
    puts "Syncing EPG data..."
    result = EPGSyncService.call
    puts "Done. Synced #{result[:synced]} programmes across #{result[:channels]} channels."
  end

  desc "Clean up expired EPG programmes (ended > 1 week ago)"
  task epg_cleanup: :environment do
    puts "Cleaning up expired EPG programmes..."
    EPGCleanupJob.new.perform
    puts "Done."
  end
end
