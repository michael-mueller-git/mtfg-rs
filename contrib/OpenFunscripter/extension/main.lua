json = require "json"
-- mtfg-rs LUA Wrappper Version 0.0.1

-- global var
processHandleMTFG = nil
status = "MTFG not running"
updateCounter = 0
scriptIdx = 1
tmpFileName = "funscript_actions.json"
tmpFileExists = false
stopAtNextActionPoint = true
filterSimilarTimestamps = true
epsilon = 10.0
persons = 1
personsOptions = {"1", "2"}
frameStepSize = 1
frameStepSizes = {"1", "2", "3", "4", "5"}
filterIdx = 1
filterNames = {
    "VR-3D-SBS-180"
}
filterValues = {
    "v360=input=he:in_stereo=sbs:pitch={pitch}:yaw={yaw}:roll=0:output=flat:d_fov={fov}:w=512:h=512"
}

function exists(file)
   return os.rename(file, file)
end

function get_platform()
    if ofs.ExtensionDir():find("^/home/") ~= nil then
        return "Linux"
    else
        return "Windows"
    end
end

platform = get_platform()

function binding.start_funscript_generator()
    if processHandleMTFG then
        print('MTFG already running')
        return
    end

    scriptIdx = ofs.ActiveIdx()
    local tmpFile = ofs.ExtensionDir() .. "/" .. tmpFileName
    local video = player.CurrentVideo()
    local script = ofs.Script(scriptIdx)
    local currentTime = player.CurrentTime()
    local fps = player.FPS()

    local next_action = nil
    if stopAtNextActionPoint then
        next_action, _ = script:closestActionAfter(currentTime)
        if next_action and next_action.at < (currentTime + 0.5) then
            next_action, _ = script:closestActionAfter(next_action.at)
        end
    end

    print("tmpFile: ", tmpFile)
    print("video: ", video)
    print("fps", fps)
    print("currentScriptIdx: ", scriptIdx)
    print("currentTime: ", currentTime)
    print("nextAction: ", next_action and tostring(next_action.at) or "nil")

    local cmd = ""
    local args = {}

    if platform == "Linux" then
        cmd = ofs.ExtensionDir() .. "/mtfg-rs/mtfg-rs.sh"
    else
        print("ERROR: Platform Not Implemented (", platform, ")")
        return
    end

    table.insert(args, "--start")
    table.insert(args, tostring(math.floor(currentTime*1000)))
    table.insert(args, "--input")
    table.insert(args, video)
    table.insert(args, "--output")
    table.insert(args, tmpFile)
    table.insert(args, "--epsilon")
    table.insert(args, tostring(epsilon))
    table.insert(args, "--step")
    table.insert(args, frameStepSize)
    table.insert(args, "--persons")
    table.insert(args, persons)
    table.insert(args, "--filter")
    table.insert(args, filterValues[filterIdx])

    if next_action then
        table.insert(args, "--end")
        table.insert(args, tostring(math.floor(next_action.at*1000.0)))
    end

    print("cmd: ", cmd)
    print("args: ", table.unpack(args))

    processHandleMTFG = Process.new(cmd, table.unpack(args))

    status = "MTFG running"
end


function import_funscript_generator_json_result()
    status = "MTFG not running"
    local tmpFile = ofs.ExtensionDir() .. "/" .. tmpFileName
    local f = io.open(tmpFile)
    if not f then
        print('Funscript Generator json output file not found')
        return
    end

    local content = f:read("*a")
    f:close()
    json_body = json.decode(content)
    actions = json_body["actions"]

    local fps = player.FPS()
    local frame_time = 1.0/fps
    print("Frame Time:", frame_time)
    local filtered = 0

    script = ofs.Script(scriptIdx)

    for _, action in pairs(actions) do
        local closest_action, _ = script:closestAction(action["at"]/1000.0)
        local new_action = Action.new(action["at"]/1000.0, action["pos"], true)
        if filterSimilarTimestamps and closest_action and math.abs(closest_action.at - new_action.at) <= frame_time then
            filtered = filtered + 1
        else
            script.actions:add(new_action)
        end
    end

    script:commit()

    if filterSimilarTimestamps then
        print('filtered timestamps', filtered)
    end

end


function init()
    print("OFS Version:", ofs.Version())
    print("Detected OS: ", platform)
end


function is_empty(s)
  return s == nil or s == ''
end


function update_tmp_file_exists()
    local tmpFile = ofs.ExtensionDir() .. "/" .. tmpFileName
    local f = io.open(tmpFile)
    if f then
        tmpFileExists = true
        f:close()
    else
        tmpFileExists = false
    end
end


function update(delta)
    updateCounter = updateCounter + 1
    if processHandleMTFG and not processHandleMTFG:alive() then
        print('funscript generator completed import result')
        processHandleMTFG = nil
        import_funscript_generator_json_result()
    end
    if math.fmod(updateCounter, 100) == 1 then
        update_tmp_file_exists()
    end
end


function gui()
    ofs.Text("Status: "..status)
    ofs.Separator()
    ofs.Text("Options:")

    ofs.Text("  o ")
    ofs.SameLine()
    persons, _ = ofs.Combo("Moving Persons", persons, personsOptions)

    ofs.Text("  o ")
    ofs.SameLine()
    frameStepSize, _ = ofs.Combo("Frame Step Size", frameStepSize, frameStepSizes)

    ofs.Text("  o ")
    ofs.SameLine()
    epsilon, _ = ofs.Slider("Epsilon", epsilon, 0.0, 100.0)

    ofs.Text("  o ")
    ofs.SameLine()
    filterIdx, _ = ofs.Combo("Video Filter", filterIdx, filterNames)

    ofs.Separator()
    ofs.Text("Action:")

    ofs.SameLine()
    if not processHandleMTFG then
        if ofs.Button("Start MTFG") then
            binding.start_funscript_generator()
        end
    else
        if ofs.Button("Kill MTFG") then
            if platform == "Linux" then
                os.execute("pkill -f mtfg-rs")
            end
        end
    end

    if tmpFileExists then
        ofs.SameLine()
        if ofs.Button("Force Import") then
            scriptIdx = ofs.ActiveIdx()
            import_funscript_generator_json_result()
        end
    end
end
