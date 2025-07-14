# Main menu
add-files = Add file(s)
toggle-play-pause = Play/Pause
stop = Stop
next-frame = Next frame
    .desc = Jump to the next video frame
previous-uri = Previous file
next-uri = Next file
toggle-editor-visibility = Show/Hide editor
toggle-playlist-visibility = Show/Hide playlist
volume = Volume
speed = Speed
increase = Increase
decrease = Decrease
reset = Reset
audio-track = Audio track
toggle-full-screen = Full screen
dump-pipeline = Dump Pipeline
video-offsets = Video offsets...
preferences-dialog = Preferences...
about = About MXL Plyr...
license-dialog = Licensing...
create-report = Create Report...
third-party-licenses = Third party licenses...
quit = Quit

close = Close
fatal-error-title = Fatal Error
error-title = Error
warning-title = Warning

# App
drop-files-to-add = Drop files here to add them to the playlist
he-is-dead-jim = He's dead, Jim!
media-files = Media files
all-files = All files
audio-stream = {$channels} channels ({$sample_rate}Hz)
audio-stream-with-language = {$channels} channels ({$sample_rate}Hz) - [{$language}]
disable = Disable
enable = Enable
dumped-pipeline = Dumped pipeline
license-active-failed-title = Failed licensing
get-failed-procs-failed-title = Failed to get failed executions
report-creation-succeeded = The report was successfully saved to '{$file_name}'

# Preferences ui
preferences = Preferences
    .general = General
    .appearance = Appearance
    .video-decoder = Video decoder
auto-play = Auto play
    .description = Auto play starts the playback automatically if the application is started with a file
color-scheme = Color scheme
    .description = Set the color scheme of the application
    .default = System
    .dark = Dark
    .light = Light

# Preferences
preferences-load-failed-title = Failed to load preferences
preferences-fallback-to-default = Falling back to default preferences

# Editor
editor-start-file = Start: File
editor-end-file = End: File
editor-set-start = Set editor start position to the current player position
editor-set-end = Set editor end position to the current player position
editor-x = X:
editor-y = Y:
editor-width = Width:
editor-height = Height:
editor-adjust-cut-area = Adjust
    .tooltip = Adjust the rectangle to the video or the allowed maximum
editor-start = Start...
    .tooltip = Start the rendering process
editor-cannot-start = Cannot start the rendering process
editor-header-state-init = Rendering - Initializing...
editor-header-state-rendering = Rendering - completed {$percent}%
editor-header-state-finished = Rendering - complete
editor-description = This process may take a long time depending on the selected image size, your computer's processing power and the length of your video.
editor-eta-init = Initializing...
editor-eta-estimating = Estimating...
editor-eta-unit-hours =
    { $hours ->
        [one] {$hours} hour
       *[other] {$hours} hours
    }
editor-eta-unit-minutes =
    { $minutes ->
        [one] {$minutes} minute
       *[other] {$minutes} minutes
    }
editor-eta-unit-seconds =
    { $seconds ->
        [one] {$seconds} second
       *[other] {$seconds} seconds
    }
editor-eta-hours-minutes = About {$hours} and {$minutes} left
editor-eta-hours = About {$hours} left
editor-eta-minutes = About {$minutes} left
editor-eta-minutes-seconds = About {$minutes} and {$seconds} left
editor-eta-seconds = About {$seconds} left
editor-eta-finished = Completed
editor-btn-cancel = _Cancel
editor-btn-ok = _Ok

# Commandline licensing:
cmd-licensing =
    .act-license-key = License key to activate
    .act-offline-request = Offline activation request file
    .act-offline-response = Offline activation response file
    .deact-offline-request = Offline deactivation request file
