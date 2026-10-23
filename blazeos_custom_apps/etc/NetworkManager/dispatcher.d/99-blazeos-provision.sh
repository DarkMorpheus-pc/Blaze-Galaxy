#!/bin/bash
if [ "$2" = "up" ]; then
    if [ -f /var/lib/blazeos/desktop-provision-pending ]; then
        systemctl start blazeos-desktop-provision.service &
    fi
fi
