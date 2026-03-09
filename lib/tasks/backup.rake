namespace :backup do
  desc "Backup the primary SQLite database to S3 (RETENTION=7, PREFIX=backups/db/)"
  task database: :environment do
    retention = (ENV["RETENTION"] || 7).to_i
    prefix = ENV.fetch("PREFIX", "backups/db/")
    result = DatabaseBackupService.call(retention: retention, prefix: prefix)
    puts "Backup uploaded: #{result[:key]} (#{number_to_human_size(result[:size])})"
  end
end

def number_to_human_size(bytes)
  if bytes >= 1_048_576
    format("%.1f MB", bytes / 1_048_576.0)
  elsif bytes >= 1024
    format("%.1f KB", bytes / 1024.0)
  else
    "#{bytes} B"
  end
end
