import { commands, hooks } from "@commands";
import { useGetTranslation } from "@hooks";
import { FileOpenRounded } from "@mui/icons-material";
import { Box, Modal } from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import { memo, useCallback, useEffect, useState } from "react";

const FileDrop = memo(function FileDrop() {
    const [isShown, setIsShown] = useState(false);

    const owmlInstalled = hooks.checkOWML("OWML_CONFIG_RELOAD")[1];

    const getTranslation = useGetTranslation();

    const onDragEnter = useCallback(() => {
        setIsShown(true);
    }, []);

    const onDragLeave = useCallback(() => {
        setIsShown(false);
    }, []);

    useEffect(() => {
        let cancel = false;
        listen("DRAG_ENTER", () => {
            if (cancel || !owmlInstalled) return;
            onDragEnter();
        });
        listen("DRAG_LEAVE", () => {
            if (cancel || !owmlInstalled) return;
            onDragLeave();
        });
        return () => {
            cancel = true;
        };
    }, [onDragLeave, onDragEnter, owmlInstalled]);

    useEffect(() => {
        commands.registerDropHandler();
    }, []);

    return (
        <Modal keepMounted open={isShown}>
            <Box
                sx={{ pointerEvents: "none" }}
                display="flex"
                alignItems="center"
                justifyContent="center"
                flexDirection="column"
                height="100%"
            >
                <FileOpenRounded fontSize="large" />
                <h2>{getTranslation("FILE_DROP_MESSAGE")}</h2>
            </Box>
        </Modal>
    );
});

export default FileDrop;
