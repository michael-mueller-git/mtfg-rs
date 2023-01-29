json = require "json"
rdp = require "RamerDouglasPeucker"
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
epsilon = 1.8
epsilonPost = 1.05
persons = "1"
personsOptions = {"1", "2"}
filterIdx = 1
currentScript = nil
filterOptions = {
    "pad='max(ih,iw):max(ih,iw):max(ih,iw)/2:max(ih,iw)/2',scale=512:512",
    "v360=input=he:in_stereo=sbs:pitch=0:yaw=0:roll=0:output=flat:d_fov=98:w=512:h=512",
    "v360=input=he:in_stereo=sbs:pitch=-25:yaw=0:roll=0:output=flat:d_fov=98:w=512:h=512"
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
        cmd = ofs.ExtensionDir() .. "/mtfg-rs"
    else
        print("ERROR: Platform Not Implemented (", platform, ")")
        return
    end

    table.insert(args, "-s")
    table.insert(args, tostring(math.floor(currentTime*1000)))
    table.insert(args, "-i")
    table.insert(args, video)
    table.insert(args, "-o")
    table.insert(args, tmpFile)
    table.insert(args, "--epsilon")
    table.insert(args, tostring(epsilon))
    table.insert(args, "-p")
    table.insert(args, persons)
    table.insert(args, "-f")
    table.insert(args, filterOptions[filterIdx])

    if next_action then
        table.insert(args, "-e")
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


table.filter = function(t, filterIter)
  local out = {}

  for idx, item in pairs(t) do
    if filterIter(item) then
        out[idx] = item
    end
  end

  return out
end

function simplify(rdpEpsilon)
    print("simplify with epsilon", rdpEpsilon)
    scriptIdx = ofs.ActiveIdx()

    if not currentScript then
        print("fetch new data")
        currentScript = ofs.Script(scriptIdx)
    end

    script = currentScript
    local selection = table.filter(script.actions, function(item) return item.selected end)
    local result = rdp(selection, rdpEpsilon, true, "at", "pos")

    for idx, action in ipairs(script.actions) do
        script:markForRemoval(idx)
    end
    script:removeMarked()

    local filtered = 0
    local fps = player.FPS()
    local frame_time = 1.0/fps
    for idx, action in ipairs(result) do
        local closest_action, _ = script:closestAction(action.at)
        local new_action = Action.new(action.at, action.pos, true)
        print("new", action.at, action.pos, new_action.at, new_action.pos)
        if closest_action then
            -- print("found closest action", action.at, closest_action.at)
            -- print("closest action diff", math.abs(closest_action.at - new_action.at), frame_time)
        end
        if filterSimilarTimestamps and closest_action and math.abs(closest_action.at - new_action.at) <= frame_time then
            filtered = filtered + 1
            -- print("filter")
        else
            script.actions:add(new_action)
            -- print("add")
        end
    end

    -- for idx, a in ipairs(script.actions) do
    --     print("content", idx, a.at)
    -- end

    if filterSimilarTimestamps then
        print('filtered timestamps', filtered)
    end

    script:commit()
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
    epsilon, _ = ofs.Slider("Epsilon", epsilon, 0.0, 100.0)

    ofs.Text("  o ")
    ofs.SameLine()
    filterIdx, _ = ofs.Combo("Video Filter", filterIdx, filterOptions)

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

    ofs.Separator()
    ofs.Text("Post-Process:")

    ofs.Text("  o ")
    ofs.SameLine()
    if ofs.Button("undo") then
        scriptIdx = ofs.ActiveIdx()
        ofs.Undo()
    end

    ofs.Text("  o ")
    ofs.SameLine()
    epsilonPost, epsilonPostChanged  = ofs.Slider("Epsilon Post-Process", epsilonPost, 0.0, 10.0)
    if epsilonPostChanged then
        simplify(epsilonPost)
    end
    ofs.SameLine()
    if ofs.Button("apply") then
        currentScript = nil
    end


end
