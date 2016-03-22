#include <stdio.h>
#include <unistd.h>
#include <pty.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/ioctl.h>
#include <string.h>

// TODO can this file re-written to Rust?

// TODO int??
int pty_fd = -1;

int main(int argc, char* argv[]) {
    char name[100] = {0};
    pid_t pid = forkpty(&pty_fd, name, NULL, NULL);

    if (pid < 0) {
        fprintf(stderr, "error: pid < 0\n");
        return 1;
    }
    else if (!pid) {

        // TODO test: terminal resize
        /* struct winsize winp = { 40, 100, 0, 0 }; */
        /* int ioctl_result = ioctl(0, TIOCSWINSZ, &winp); */
        /* if (ioctl_result) { */
        /*     int no = errno; */
        /*     fprintf(stderr, "ioctl error: %d errno %d (%s)\n", ioctl_result, no, strerror(no)); */
        /* } */

        /* ioctl_result = ioctl(0, TIOCGWINSZ, &winp); */
        /* if (ioctl_result) { */
        /*     int no = errno; */
        /*     fprintf(stderr, "ioctl error: %d errno %d (%s)\n", ioctl_result, no, strerror(no)); */
        /* } */
        /* fprintf(stderr, "slave: window size: %d %d\n", winp.ws_row, winp.ws_col); */

        // TODO as argument
        char* args[] = {"/bin/bash", "-c", "/bin/ls --color -l", NULL};
        int exe_result = execvp(args[0], args);
        if (exe_result) {
            fprintf(stderr, "cannot execute shell: %d\n", exe_result);
        }
        return 0;
    } else {
        // parent
    }

    // TODO test: terminal size
    struct winsize winp = {0};
    // TIOCSWINSZ set
    // TIOCGWINSZ get
    int ioctl_result = ioctl(pty_fd, TIOCGWINSZ, &winp);
    if (ioctl_result) {
        fprintf(stderr, "ioctl error: %d\n", ioctl_result);
    }
    fprintf(stderr, "master: window size: %d %d\n", winp.ws_row, winp.ws_col);


    /* char ptrname_buf[100] = {0}; */
    /* int ret = ptsname_r(pty_fd, ptrname_buf, sizeof(ptrname_buf)); */

    fcntl(STDIN_FILENO, F_SETFL, O_NONBLOCK);
    fcntl(pty_fd, F_SETFL, O_NONBLOCK);

    static unsigned char buf[1024] = {0};
    while (1) {
        fprintf(stderr, "while(1)\n");
        struct timeval timeout = {0, 100000000};
        struct timeval* timeout_p = NULL;
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(STDIN_FILENO, &fds);
        FD_SET(pty_fd, &fds);
        if (select(pty_fd + 1, &fds, 0, 0, &timeout) > 0) {
            if (FD_ISSET(pty_fd, &fds)) {
                // pty slave

                // TODO int??
                int read_len = read(pty_fd, buf, sizeof(buf));
                if (read_len < 0) {
                    break;
                }
                write(STDOUT_FILENO, buf, read_len);
                fprintf(stderr, "transmitted %d bytes\n", read_len);
            } else {
                // stdin from user
                // just pass user input to terminal
                // TODO how to implement ioctl request

                // TODO int??
                int read_len = read(STDIN_FILENO, buf, sizeof(buf));
                if (read_len < 0) {
                    // TODO ??
                    break;
                }
                write(pty_fd, buf, read_len);
            }
        }
    }

    return 0;
}
