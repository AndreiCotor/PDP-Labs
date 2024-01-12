import message.ActionMessage;
import mpi.MPI;

import java.util.Comparator;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.PriorityBlockingQueue;
import java.util.concurrent.atomic.AtomicInteger;

public class DSMFrameworkNotifyBuffer {

    private final Map<String, PriorityBlockingQueue<ActionMessage>> minHeapBuffer;
    private final AtomicInteger timestamp;

    public DSMFrameworkNotifyBuffer() {
        Comparator<ActionMessage> comparator = (m1, m2) -> {
            if (m1.getTimestamp() != m2.getTimestamp()) {
                return Integer.compare(m1.getTimestamp(), m2.getTimestamp());
            } else {
                return Integer.compare(m1.getSource(), m2.getSource());
            }
        };

        minHeapBuffer = new HashMap<>();
        minHeapBuffer.put("x", new PriorityBlockingQueue<>(10, comparator));
        minHeapBuffer.put("y", new PriorityBlockingQueue<>(10, comparator));
        minHeapBuffer.put("z", new PriorityBlockingQueue<>(10, comparator));

        this.timestamp = new AtomicInteger(0);
    }

    public void add(ActionMessage actionMessage) {
        minHeapBuffer.get(actionMessage.getVariable()).add(actionMessage);
    }

    public void flush(String variable, int timestamp) {
        while (!minHeapBuffer.get(variable).isEmpty() && minHeapBuffer.get(variable).peek().getTimestamp() < timestamp) {
            ActionMessage message = minHeapBuffer.get(variable).poll();
            System.out.println("Notified process " + MPI.COMM_WORLD.Rank() + " with " + message.toString());
        }
    }

    public void flushAll() {
        while (!minHeapBuffer.get("x").isEmpty()) {
            ActionMessage message = minHeapBuffer.get("x").poll();
            System.out.println("Notified process " + MPI.COMM_WORLD.Rank() + " with " + message.toString());
        }

        while (!minHeapBuffer.get("y").isEmpty()) {
            ActionMessage message = minHeapBuffer.get("y").poll();
            System.out.println("Notified process " + MPI.COMM_WORLD.Rank() + " with " + message.toString());
        }

        while (!minHeapBuffer.get("z").isEmpty()) {
            ActionMessage message = minHeapBuffer.get("z").poll();
            System.out.println("Notified process " + MPI.COMM_WORLD.Rank() + " with " + message.toString());
        }
    }

    public int getTimestamp() {
        return timestamp.get();
    }

    public void updateTimestamp(int value) {
        timestamp.getAndUpdate(el -> Math.max(el, value));
    }
}
