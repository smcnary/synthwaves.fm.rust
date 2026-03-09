namespace :recovery do
  desc "Recover orphaned S3 files into the database (USER_EMAIL=x COMMIT=true for real run)"
  task from_s3: :environment do
    user_email = ENV.fetch("USER_EMAIL") { abort "USER_EMAIL is required" }
    commit = ENV["COMMIT"] == "true"

    puts commit ? "Running in COMMIT mode — records will be created" : "Running in DRY-RUN mode (pass COMMIT=true to write)"
    puts

    stats = S3RecoveryService.call(user_email: user_email, commit: commit)

    puts
    puts "Summary:"
    puts "  Scanned:       #{stats[:scanned]}"
    puts "  Audio created: #{stats[:audio_created]}"
    puts "  Video created: #{stats[:video_created]}"
    puts "  Existing:      #{stats[:existing]}"
    puts "  Skipped:       #{stats[:skipped]}"
    puts "  Errors:        #{stats[:errors]}"
  end
end
