require 'octokit'
require 'tmpdir'
require 'fileutils'

REPO_OWNER = 'esrlabs'
REPO_NAME = 'chipmunk'

RAKE_COMMANDS = [
  'rake clobber',
  'rake bindings:build',
  'rake bindings:build_spec',
  'rake bindings:test:stream',
  'rake bindings:test:indexes',
  'rake bindings:test:search',
  'rake bindings:test:observe'
]

SHELL_SCRIPT_PATH = 'application/apps/rustcore/ts-bindings/spec'

if ARGV.length > 1
  puts "Usage: ruby scripts/tools/run_benchmarks.rb <number_of_releases>"
  exit(1)
end

NUMBER_OF_RELEASES = ARGV[0].to_i == 0 ? 1 : ARGV[0].to_i

puts "running benchmarks for last #{NUMBER_OF_RELEASES} releases"

client = Octokit::Client.new()

# Fetch the latest releases from the GitHub repository
releases = client.releases("#{REPO_OWNER}/#{REPO_NAME}")

# Iterate over the specified number of releases
releases.take(NUMBER_OF_RELEASES).each_with_index do |release, index|
  puts "Processing release #{index + 1}: #{release.name}"

  ENV_VARS = {
    'JASMIN_TEST_CONFIGURATION' => './spec/benchmarks.json',
    'PERFORMANCE_RESULTS_FOLDER' => 'chipmunk_performance_results',
    'PERFORMANCE_RESULTS' => "Benchmark_#{release.tag_name}.json",
    'SH_HOME_DIR' => "/chipmunk"
  }

  # Create a temporary directory for this release
  Dir.mktmpdir do |temp_dir|
    begin
      # Clone the repository into the temporary directory
      system("git clone --depth 1 --branch #{release.tag_name} https://github.com/#{REPO_OWNER}/#{REPO_NAME}.git #{temp_dir}")

      # Copy scripts folder to have the test cases available in the cloned repo
      FileUtils.cp_r("#{SHELL_SCRIPT_PATH}/.", "#{temp_dir}/#{SHELL_SCRIPT_PATH}/.", verbose: true)

      result_path = "#{ENV_VARS['SH_HOME_DIR']}/#{ENV_VARS['PERFORMANCE_RESULTS_FOLDER']}/Benchmark_#{release.tag_name}.json"

      # Change directory to the temporary directory
      Dir.chdir(temp_dir) do
        # Execute the shell script
        ENV_VARS.each do |key, value|
          ENV[key] = "#{value}"
        end

        system("corepack enable")
        system("yarn cache clean")

        if File.exist?("#{SHELL_SCRIPT_PATH}/#{ENV_VARS['JASMIN_TEST_CONFIGURATION'].gsub('./spec/', '')}")
          puts "File exists."
        else
          break
        end

        system("printenv")

        if File.exist?(result_path)
          FileUtils.rm(result_path, verbose: true)
        end

        # Run each Rake command
        RAKE_COMMANDS.each do |command|
          puts "Running #{command} for tag #{release.name}"
          system(command)
        end
      end

      if File.exist?(result_path)
        puts "Benchmark results:"
        system("cat #{result_path}")
      else
        puts "Benchmark results not found at #{result_path}."
      end

    rescue => e
      puts "An error occurred while processing release #{release.tag_name}: #{e.message}"
    end

    puts "Completed processing release #{index + 1}: #{release.name}"
  end
end

# Method to read and parse JSON files
def read_benchmark_data(file_path)
  puts "Data in file = #{file_path}\n#{File.read(file_path)}\n:: EOF"
  { file_name: File.basename(file_path), data: JSON.parse(File.read(file_path)) }
end

# Method to collect data from the latest 10 JSON files
def collect_latest_benchmark_data(directory)
  Dir.glob("#{directory}/Benchmark*.json").map do |file|
    read_benchmark_data(file)
  end
end

# Method to generate graphs for each performance test type
def generate_performance_graphs(data)
  # Hash to store data organized by test name
  test_data = {}

  # Collect data by test name
  data.each do |benchmark|
    benchmark[:data].each do |entry|
      test_name = entry['name']
      actual_value = entry['actual']
      file_name = benchmark[:file_name]

      if test_data[test_name]
        test_data[test_name] << { release: file_name.gsub("Benchmark_","").gsub(".json",""), actual_value: actual_value }
      else
        test_data[test_name] = [{ release: file_name.gsub("Benchmark_","").gsub(".json",""), actual_value: actual_value }]
      end
    end
  end

  puts ("Test data = #{test_data.to_json}")
  File.open("#{ENV_VARS['SH_HOME_DIR']}/#{ENV_VARS['PERFORMANCE_RESULTS_FOLDER']}/data.json", 'w') do |file|
    file.write(test_data.to_json)
  end
  puts "Benchmark data created successfully!"
end

benchmark_data = collect_latest_benchmark_data("#{ENV_VARS['SH_HOME_DIR']}/#{ENV_VARS['PERFORMANCE_RESULTS_FOLDER']}")
generate_performance_graphs(benchmark_data)

