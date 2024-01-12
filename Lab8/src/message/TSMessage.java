package message;

public class TSMessage extends BaseMessage {
    private int timestamp;

    public TSMessage(int timestamp) {
        this.timestamp = timestamp;
    }

    public int getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(int timestamp) {
        this.timestamp = timestamp;
    }
}
