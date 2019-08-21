# close all open windows gracefully without closing the Desktop environment
WIN_IDs=$(wmctrl -l | grep -vwE "Desktop$" | cut -f1 -d' ')
for i in $WIN_IDs
do
   wmctrl -ic "$i"
done


# Keep checking and waiting until all windows are closed
while test $WIN_IDs
do
    sleep 0.1;
    WIN_IDs=$(wmctrl -l | grep -vwE "Desktop$" | cut -f1 -d' ')
done

