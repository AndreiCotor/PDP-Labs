import mpi.MPI;

import java.util.*;

public class Main {
    public static void main(String[] args) throws InterruptedException {
        MPI.Init(args);
        int me = MPI.COMM_WORLD.Rank();
        int size = MPI.COMM_WORLD.Size();

        Map<String, Set<Integer>> subscriptions = new HashMap<>();
        subscriptions.put("x", new HashSet<>(Arrays.asList(0, 1, 2)));
        subscriptions.put("y", new HashSet<>(Arrays.asList(0, 2)));
        subscriptions.put("z", new HashSet<>(Arrays.asList(1, 2)));

        DSMFrameworkNotifyBuffer dsmFrameworkNotifyBuffer = new DSMFrameworkNotifyBuffer();
        DSMFramework dsm = new DSMFramework(subscriptions, dsmFrameworkNotifyBuffer);

        System.out.println("Start " + me + " of " + size);
        if (me == 0) {
            Thread thread = new Thread(new DSMFrameworkListener(dsmFrameworkNotifyBuffer));
            thread.start();
            dsm.set("x", 10);
            dsm.set("y", 30);
            dsm.set("z", 20);

            Thread.sleep(2000);
            dsm.close();
            thread.join();
        } else if (me == 1) {
            Thread thread = new Thread(new DSMFrameworkListener(dsmFrameworkNotifyBuffer));
            thread.start();
            dsm.set("x", 15);
            dsm.compareAndExchange("x", 15, 25);
            thread.join();
        } else if (me == 2) {
            Thread thread = new Thread(new DSMFrameworkListener(dsmFrameworkNotifyBuffer));
            thread.start();
            dsm.set("z", 50);
            dsm.compareAndExchange("z", 50, 30);
            thread.join();
        }

        MPI.Finalize();
    }
}