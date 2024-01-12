import message.*;

import java.util.Map;
import java.util.Set;

import mpi.MPI;

public class DSMFramework {
    private final Map<String, Set<Integer>> subscriberConfig;
    private final DSMFrameworkNotifyBuffer notifyBuffer;

    public DSMFramework(Map<String, Set<Integer>> subscriberConfig, DSMFrameworkNotifyBuffer notifyBuffer) {
        this.subscriberConfig = subscriberConfig;
        this.notifyBuffer = notifyBuffer;
    }

    public void set(String variable, int value) {
        sendMessageToSubscribers(variable, new PrepMessage(variable, notifyBuffer.getTimestamp()));
        int ts = 1 + receiveTimestampFromSubscribers(variable);

        notifyBuffer.flush(variable, ts);
        notifyBuffer.updateTimestamp(ts);

        ActionMessage msg = new ActionMessage("set", variable, value, -1, ts, MPI.COMM_WORLD.Rank());
        sendMessageToSubscribers(variable, msg);

        if (subscriberConfig.get(variable).contains(MPI.COMM_WORLD.Rank())) {
            notifyBuffer.add(msg);
        }
    }

    public void compareAndExchange(String variable, int prev, int after) {
        sendMessageToSubscribers(variable, new PrepMessage(variable, notifyBuffer.getTimestamp()));
        int ts = 1 + receiveTimestampFromSubscribers(variable);

        notifyBuffer.flush(variable, ts);
        notifyBuffer.updateTimestamp(ts);

        ActionMessage msg = new ActionMessage("exchange", variable, after, prev, ts, MPI.COMM_WORLD.Rank());
        sendMessageToSubscribers(variable, msg);

        if (subscriberConfig.get(variable).contains(MPI.COMM_WORLD.Rank())) {
            notifyBuffer.add(msg);
        }
    }

    private void sendMessageToSubscribers(String variable, BaseMessage message) {
        for (var subscriber: subscriberConfig.get(variable)) {
            if (MPI.COMM_WORLD.Rank() != subscriber) {
                MPI.COMM_WORLD.Send(new Object[]{message}, 0, 1, MPI.OBJECT, subscriber, 0);
            }
        }
    }

    private int receiveTimestampFromSubscribers(String variable) {
        int result = notifyBuffer.getTimestamp();
        for (var subscriber: subscriberConfig.get(variable)) {
            if (MPI.COMM_WORLD.Rank() != subscriber) {
                Object[] messageObjects = new Object[1];;
                MPI.COMM_WORLD.Recv(messageObjects, 0, 1, MPI.OBJECT, subscriber, 1);

                TSMessage tsMessage = (TSMessage) messageObjects[0];
                result = Math.max(result, tsMessage.getTimestamp());
            }
        }
        return result;
    }

    private void sendMessageToAll(BaseMessage message) {
        for (int i = 0; i < MPI.COMM_WORLD.Size(); i++) {
            MPI.COMM_WORLD.Send(new Object[]{message}, 0, 1, MPI.OBJECT, i, 0);
        }
    }

    public void close() {
        this.sendMessageToAll(new CloseMessage());
    }
}