# Windows installer

The MVP installer target is the current user's `%LOCALAPPDATA%\GlucoseDashboard`
directory. Release packaging should place the backend executable and `frontend/dist`
there, then add the executable directory to the user's PATH without overwriting the
configuration file.
